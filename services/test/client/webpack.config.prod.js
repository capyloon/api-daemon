/* eslint-disable */

'use strict'

const path = require('path');
const webpack = require('webpack');
const HtmlWebpackPlugin = require('html-webpack-plugin');

let config = require('./webpack.config');


const UglifyJsPlugin = require("uglifyes-webpack-plugin");

const uglifyOptions = {
    sourceMap: false,
    beautify: false,
    comments: false,
    ecma: 8,
    compress: {
        collapse_vars: true,
        drop_console: true,
        screw_ie8: true,
        warnings: false
    },
    mangle: {
        screw_ie8: true,
        except: ['$super', '$', 'exports', 'require']
    },
    output: {
        screw_ie8: true,
        comments: false
    }
};

config = Object.assign({}, config);


config.plugins = [
    new webpack.DefinePlugin({
        'process.env': {
            'NODE_ENV': JSON.stringify('production')
        }
    }),
    new UglifyJsPlugin({
        uglifyOptions: {
            ecma: 8
        },
    }),
];

module.exports = config;