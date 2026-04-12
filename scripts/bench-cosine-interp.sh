#!/bin/bash
cd profiling
cargo bench --bench time_warp_bench -- cosine_interp --sample-size 1000
