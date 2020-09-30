/* eslint-disable */

'use strict'

const webpack = require('webpack');
let config = require('./webpack.config');

config = Object.assign({}, config);


config.plugins = [
    new webpack.DefinePlugin({
        'process.env': {
            'NODE_ENV': JSON.stringify('production')
        }
    }),

];

module.exports = config;
