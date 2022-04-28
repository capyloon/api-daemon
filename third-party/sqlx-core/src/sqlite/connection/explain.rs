use crate::error::Error;
use crate::from_row::FromRow;
use crate::sqlite::connection::{execute, ConnectionState};
use crate::sqlite::type_info::DataType;
use crate::sqlite::SqliteTypeInfo;
use crate::HashMap;
use std::str::from_utf8;

// affinity
const SQLITE_AFF_NONE: u8 = 0x40; /* '@' */
const SQLITE_AFF_BLOB: u8 = 0x41; /* 'A' */
const SQLITE_AFF_TEXT: u8 = 0x42; /* 'B' */
const SQLITE_AFF_NUMERIC: u8 = 0x43; /* 'C' */
const SQLITE_AFF_INTEGER: u8 = 0x44; /* 'D' */
const SQLITE_AFF_REAL: u8 = 0x45; /* 'E' */

// opcodes
const OP_INIT: &str = "Init";
const OP_GOTO: &str = "Goto";
const OP_COLUMN: &str = "Column";
const OP_MAKE_RECORD: &str = "MakeRecord";
const OP_INSERT: &str = "Insert";
const OP_IDX_INSERT: &str = "IdxInsert";
const OP_OPEN_READ: &str = "OpenRead";
const OP_OPEN_WRITE: &str = "OpenWrite";
const OP_OPEN_EPHEMERAL: &str = "OpenEphemeral";
const OP_OPEN_AUTOINDEX: &str = "OpenAutoindex";
const OP_AGG_STEP: &str = "AggStep";
const OP_FUNCTION: &str = "Function";
const OP_MOVE: &str = "Move";
const OP_COPY: &str = "Copy";
const OP_SCOPY: &str = "SCopy";
const OP_NULL_ROW: &str = "NullRow";
const OP_INT_COPY: &str = "IntCopy";
const OP_CAST: &str = "Cast";
const OP_STRING8: &str = "String8";
const OP_INT64: &str = "Int64";
const OP_INTEGER: &str = "Integer";
const OP_REAL: &str = "Real";
const OP_NOT: &str = "Not";
const OP_BLOB: &str = "Blob";
const OP_VARIABLE: &str = "Variable";
const OP_COUNT: &str = "Count";
const OP_ROWID: &str = "Rowid";
const OP_NEWROWID: &str = "NewRowid";
const OP_OR: &str = "Or";
const OP_AND: &str = "And";
const OP_BIT_AND: &str = "BitAnd";
const OP_BIT_OR: &str = "BitOr";
const OP_SHIFT_LEFT: &str = "ShiftLeft";
const OP_SHIFT_RIGHT: &str = "ShiftRight";
const OP_ADD: &str = "Add";
const OP_SUBTRACT: &str = "Subtract";
const OP_MULTIPLY: &str = "Multiply";
const OP_DIVIDE: &str = "Divide";
const OP_REMAINDER: &str = "Remainder";
const OP_CONCAT: &str = "Concat";
const OP_RESULT_ROW: &str = "ResultRow";

#[derive(Debug, Clone, Eq, PartialEq)]
enum RegDataType {
    Single(DataType),
    Record(Vec<DataType>),
}

impl RegDataType {
    fn map_to_datatype(self) -> DataType {
        match self {
            RegDataType::Single(d) => d,
            RegDataType::Record(_) => DataType::Null, //If we're trying to coerce to a regular Datatype, we can assume a Record is invalid for the context
        }
    }
}

#[allow(clippy::wildcard_in_or_patterns)]
fn affinity_to_type(affinity: u8) -> DataType {
    match affinity {
        SQLITE_AFF_BLOB => DataType::Blob,
        SQLITE_AFF_INTEGER => DataType::Int64,
        SQLITE_AFF_NUMERIC => DataType::Numeric,
        SQLITE_AFF_REAL => DataType::Float,
        SQLITE_AFF_TEXT => DataType::Text,

        SQLITE_AFF_NONE | _ => DataType::Null,
    }
}

#[allow(clippy::wildcard_in_or_patterns)]
fn opcode_to_type(op: &str) -> DataType {
    match op {
        OP_REAL => DataType::Float,
        OP_BLOB => DataType::Blob,
        OP_AND | OP_OR => DataType::Bool,
        OP_ROWID | OP_COUNT | OP_INT64 | OP_INTEGER => DataType::Int64,
        OP_STRING8 => DataType::Text,
        OP_COLUMN | _ => DataType::Null,
    }
}

fn root_block_columns(
    conn: &mut ConnectionState,
) -> Result<HashMap<i64, HashMap<i64, DataType>>, Error> {
    let table_block_columns: Vec<(i64, i64, String)> = execute::iter(
        conn,
        "SELECT s.rootpage, col.cid as colnum, col.type
         FROM sqlite_schema s
         JOIN pragma_table_info(s.name) AS col
         WHERE s.type = 'table'",
        None,
        false,
    )?
    .filter_map(|res| res.map(|either| either.right()).transpose())
    .map(|row| FromRow::from_row(&row?))
    .collect::<Result<Vec<_>, Error>>()?;

    let index_block_columns: Vec<(i64, i64, String)> = execute::iter(
        conn,
        "SELECT s.rootpage, idx.seqno as colnum, col.type
         FROM sqlite_schema s
         JOIN pragma_index_info(s.name) AS idx
         LEFT JOIN pragma_table_info(s.tbl_name) as col
           ON col.cid = idx.cid
           WHERE s.type = 'index'",
        None,
        false,
    )?
    .filter_map(|res| res.map(|either| either.right()).transpose())
    .map(|row| FromRow::from_row(&row?))
    .collect::<Result<Vec<_>, Error>>()?;

    let mut row_info: HashMap<i64, HashMap<i64, DataType>> = HashMap::new();
    for (block, colnum, datatype) in table_block_columns {
        let row_info = row_info.entry(block).or_default();
        row_info.insert(colnum, datatype.parse().unwrap_or(DataType::Null));
    }
    for (block, colnum, datatype) in index_block_columns {
        let row_info = row_info.entry(block).or_default();
        row_info.insert(colnum, datatype.parse().unwrap_or(DataType::Null));
    }

    return Ok(row_info);
}

// Opcode Reference: https://sqlite.org/opcode.html
pub(super) fn explain(
    conn: &mut ConnectionState,
    query: &str,
) -> Result<(Vec<SqliteTypeInfo>, Vec<Option<bool>>), Error> {
    // Registers
    let mut r = HashMap::<i64, RegDataType>::with_capacity(6);
    // Map between pointer and register
    let mut r_cursor = HashMap::<i64, Vec<i64>>::with_capacity(6);
    // Rows that pointers point to
    let mut p = HashMap::<i64, HashMap<i64, DataType>>::with_capacity(6);

    // Nullable columns
    let mut n = HashMap::<i64, bool>::with_capacity(6);

    let root_block_cols = root_block_columns(conn)?;

    let program: Vec<(i64, String, i64, i64, i64, Vec<u8>)> =
        execute::iter(conn, &format!("EXPLAIN {}", query), None, false)?
            .filter_map(|res| res.map(|either| either.right()).transpose())
            .map(|row| FromRow::from_row(&row?))
            .collect::<Result<Vec<_>, Error>>()?;

    let mut program_i = 0;
    let program_size = program.len();
    let mut visited = vec![false; program_size];

    let mut output = Vec::new();
    let mut nullable = Vec::new();

    let mut result = None;

    while program_i < program_size {
        if visited[program_i] {
            program_i += 1;
            continue;
        }
        let (_, ref opcode, p1, p2, p3, ref p4) = program[program_i];

        match &**opcode {
            OP_INIT => {
                // start at <p2>
                visited[program_i] = true;
                program_i = p2 as usize;
                continue;
            }

            OP_GOTO => {
                // goto <p2>
                visited[program_i] = true;
                program_i = p2 as usize;
                continue;
            }

            OP_COLUMN => {
                //Get the row stored at p1, or NULL; get the column stored at p2, or NULL
                if let Some(record) = p.get(&p1) {
                    if let Some(col) = record.get(&p2) {
                        // insert into p3 the datatype of the col
                        r.insert(p3, RegDataType::Single(*col));
                        // map between pointer p1 and register p3
                        r_cursor.entry(p1).or_default().push(p3);
                    } else {
                        r.insert(p3, RegDataType::Single(DataType::Null));
                    }
                } else {
                    r.insert(p3, RegDataType::Single(DataType::Null));
                }
            }

            OP_MAKE_RECORD => {
                // p3 = Record([p1 .. p1 + p2])
                let mut record = Vec::with_capacity(p2 as usize);
                for reg in p1..p1 + p2 {
                    record.push(
                        r.get(&reg)
                            .map(|d| d.clone().map_to_datatype())
                            .unwrap_or(DataType::Null),
                    );
                }
                r.insert(p3, RegDataType::Record(record));
            }

            OP_INSERT | OP_IDX_INSERT => {
                if let Some(RegDataType::Record(record)) = r.get(&p2) {
                    if let Some(row) = p.get_mut(&p1) {
                        // Insert the record into wherever pointer p1 is
                        *row = (0..).zip(record.iter().copied()).collect();
                    }
                }
                //Noop if the register p2 isn't a record, or if pointer p1 does not exist
            }

            OP_OPEN_READ | OP_OPEN_WRITE | OP_OPEN_EPHEMERAL | OP_OPEN_AUTOINDEX => {
                //Create a new pointer which is referenced by p1

                //Create a new pointer which is referenced by p1, take column metadata from db schema if found
                if p3 == 0 {
                    if let Some(columns) = root_block_cols.get(&p2) {
                        p.insert(
                            p1,
                            columns
                                .iter()
                                .map(|(&colnum, &datatype)| (colnum, datatype))
                                .collect(),
                        );
                    } else {
                        p.insert(p1, HashMap::with_capacity(6));
                    }
                } else {
                    p.insert(p1, HashMap::with_capacity(6));
                }
            }

            OP_VARIABLE => {
                // r[p2] = <value of variable>
                r.insert(p2, RegDataType::Single(DataType::Null));
                n.insert(p3, true);
            }

            OP_FUNCTION => {
                // r[p1] = func( _ )
                match from_utf8(p4).map_err(Error::protocol)? {
                    "last_insert_rowid(0)" => {
                        // last_insert_rowid() -> INTEGER
                        r.insert(p3, RegDataType::Single(DataType::Int64));
                        n.insert(p3, n.get(&p3).copied().unwrap_or(false));
                    }

                    _ => {}
                }
            }

            OP_NULL_ROW => {
                // all registers that map to cursor X are potentially nullable
                for register in &r_cursor[&p1] {
                    n.insert(*register, true);
                }
            }

            OP_AGG_STEP => {
                let p4 = from_utf8(p4).map_err(Error::protocol)?;

                if p4.starts_with("count(") {
                    // count(_) -> INTEGER
                    r.insert(p3, RegDataType::Single(DataType::Int64));
                    n.insert(p3, n.get(&p3).copied().unwrap_or(false));
                } else if let Some(v) = r.get(&p2).cloned() {
                    // r[p3] = AGG ( r[p2] )
                    r.insert(p3, v);
                    let val = n.get(&p2).copied().unwrap_or(true);
                    n.insert(p3, val);
                }
            }

            OP_CAST => {
                // affinity(r[p1])
                if let Some(v) = r.get_mut(&p1) {
                    *v = RegDataType::Single(affinity_to_type(p2 as u8));
                }
            }

            OP_COPY | OP_MOVE | OP_SCOPY | OP_INT_COPY => {
                // r[p2] = r[p1]
                if let Some(v) = r.get(&p1).cloned() {
                    r.insert(p2, v);

                    if let Some(null) = n.get(&p1).copied() {
                        n.insert(p2, null);
                    }
                }
            }

            OP_OR | OP_AND | OP_BLOB | OP_COUNT | OP_REAL | OP_STRING8 | OP_INTEGER | OP_ROWID
            | OP_NEWROWID => {
                // r[p2] = <value of constant>
                r.insert(p2, RegDataType::Single(opcode_to_type(&opcode)));
                n.insert(p2, n.get(&p2).copied().unwrap_or(false));
            }

            OP_NOT => {
                // r[p2] = NOT r[p1]
                if let Some(a) = r.get(&p1).cloned() {
                    r.insert(p2, a);
                    let val = n.get(&p1).copied().unwrap_or(true);
                    n.insert(p2, val);
                }
            }

            OP_BIT_AND | OP_BIT_OR | OP_SHIFT_LEFT | OP_SHIFT_RIGHT | OP_ADD | OP_SUBTRACT
            | OP_MULTIPLY | OP_DIVIDE | OP_REMAINDER | OP_CONCAT => {
                // r[p3] = r[p1] + r[p2]
                match (r.get(&p1).cloned(), r.get(&p2).cloned()) {
                    (Some(a), Some(b)) => {
                        r.insert(
                            p3,
                            if matches!(a, RegDataType::Single(DataType::Null)) {
                                b
                            } else {
                                a
                            },
                        );
                    }

                    (Some(v), None) => {
                        r.insert(p3, v);
                    }

                    (None, Some(v)) => {
                        r.insert(p3, v);
                    }

                    _ => {}
                }

                match (n.get(&p1).copied(), n.get(&p2).copied()) {
                    (Some(a), Some(b)) => {
                        n.insert(p3, a || b);
                    }

                    _ => {}
                }
            }

            OP_RESULT_ROW => {
                // the second time we hit ResultRow we short-circuit and get out
                if result.is_some() {
                    break;
                }

                // output = r[p1 .. p1 + p2]
                output.reserve(p2 as usize);
                nullable.reserve(p2 as usize);

                result = Some(p1..p1 + p2);
            }

            _ => {
                // ignore unsupported operations
                // if we fail to find an r later, we just give up
            }
        }

        visited[program_i] = true;
        program_i += 1;
    }

    if let Some(result) = result {
        for i in result {
            output.push(SqliteTypeInfo(
                r.remove(&i)
                    .map(|d| d.map_to_datatype())
                    .unwrap_or(DataType::Null),
            ));
            nullable.push(n.remove(&i));
        }
    }

    Ok((output, nullable))
}

#[test]
fn test_root_block_columns_has_types() {
    use crate::sqlite::SqliteConnectOptions;
    use std::str::FromStr;
    let conn_options = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();
    let mut conn = super::EstablishParams::from_options(&conn_options)
        .unwrap()
        .establish()
        .unwrap();

    assert!(execute::iter(
        &mut conn,
        r"CREATE TABLE t(a INTEGER PRIMARY KEY, b_null TEXT NULL, b TEXT NOT NULL);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(
        execute::iter(&mut conn, r"CREATE INDEX i1 on t (a,b_null);", None, false)
            .unwrap()
            .next()
            .is_some()
    );
    assert!(execute::iter(
        &mut conn,
        r"CREATE UNIQUE INDEX i2 on t (a,b_null);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(execute::iter(
        &mut conn,
        r"CREATE TABLE t2(a INTEGER, b_null NUMERIC NULL, b NUMERIC NOT NULL);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(execute::iter(
        &mut conn,
        r"CREATE INDEX t2i1 on t2 (a,b_null);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());
    assert!(execute::iter(
        &mut conn,
        r"CREATE UNIQUE INDEX t2i2 on t2 (a,b);",
        None,
        false
    )
    .unwrap()
    .next()
    .is_some());

    let table_block_nums: HashMap<String, i64> = execute::iter(
        &mut conn,
        r"select name, rootpage from sqlite_master",
        None,
        false,
    )
    .unwrap()
    .filter_map(|res| res.map(|either| either.right()).transpose())
    .map(|row| FromRow::from_row(row.as_ref().unwrap()))
    .collect::<Result<HashMap<_, _>, Error>>()
    .unwrap();

    let root_block_cols = root_block_columns(&mut conn).unwrap();

    assert_eq!(6, root_block_cols.len());

    //prove that we have some information for each table & index
    for blocknum in table_block_nums.values() {
        assert!(root_block_cols.contains_key(blocknum));
    }

    //prove that each block has the correct information
    {
        let blocknum = table_block_nums["t"];
        assert_eq!((DataType::Int64), root_block_cols[&blocknum][&0]);
        assert_eq!((DataType::Text), root_block_cols[&blocknum][&1]);
        assert_eq!((DataType::Text), root_block_cols[&blocknum][&2]);
    }

    {
        let blocknum = table_block_nums["i1"];
        assert_eq!((DataType::Int64), root_block_cols[&blocknum][&0]);
        assert_eq!((DataType::Text), root_block_cols[&blocknum][&1]);
    }

    {
        let blocknum = table_block_nums["i2"];
        assert_eq!((DataType::Int64), root_block_cols[&blocknum][&0]);
        assert_eq!((DataType::Text), root_block_cols[&blocknum][&1]);
    }

    {
        let blocknum = table_block_nums["t2"];
        assert_eq!((DataType::Int64), root_block_cols[&blocknum][&0]);
        assert_eq!((DataType::Null), root_block_cols[&blocknum][&1]);
        assert_eq!((DataType::Null), root_block_cols[&blocknum][&2]);
    }

    {
        let blocknum = table_block_nums["t2i1"];
        assert_eq!((DataType::Int64), root_block_cols[&blocknum][&0]);
        assert_eq!((DataType::Null), root_block_cols[&blocknum][&1]);
    }

    {
        let blocknum = table_block_nums["t2i2"];
        assert_eq!((DataType::Int64), root_block_cols[&blocknum][&0]);
        assert_eq!((DataType::Null), root_block_cols[&blocknum][&1]);
    }
}
