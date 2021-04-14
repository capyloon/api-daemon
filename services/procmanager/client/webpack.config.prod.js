/* eslint-disable */

'use strict'

const webpack = require('webpack');
const UglifyJsPlugin = require('uglifyes-webpack-plugin');

let config = require('./webpack.config');

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
