set(CMAKE_SYSTEM_NAME Windows)

set(CMAKE_C_COMPILER_TARGET i686)
set(CMAKE_CXX_COMPILER_TARGET i686)
set(CMAKE_RC_COMPILER_TARGET i686)

set(CMAKE_CROSSCOMPILING TRUE)
set(CMAKE_CROSSCOMPILING_EMULATOR wine)

set(CMAKE_FIND_ROOT_PATH /usr/i686-w64-mingw32)
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)

find_program(C_COMPILER_FULL_PATH NAMES ${CMAKE_C_COMPILER_TARGET}-w64-mingw32-gcc)
if(NOT C_COMPILER_FULL_PATH)
    message(FATAL_ERROR "Cross-compiler for ${CMAKE_C_COMPILER_TARGET} not found")
endif()
set(CMAKE_C_COMPILER ${C_COMPILER_FULL_PATH})

find_program(CXX_COMPILER_FULL_PATH NAMES ${CMAKE_CXX_COMPILER_TARGET}-w64-mingw32-g++)
if(CXX_COMPILER_FULL_PATH)
    set(CMAKE_CXX_COMPILER ${CXX_COMPILER_FULL_PATH})
endif()

find_program(RC_COMPILER_FULL_PATH NAMES ${CMAKE_RC_COMPILER_TARGET}-w64-mingw32-windres)
if(RC_COMPILER_FULL_PATH)
    set(CMAKE_RC_COMPILER ${RC_COMPILER_FULL_PATH})
endif()
