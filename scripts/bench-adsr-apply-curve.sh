#!/bin/bash
cd profiling
cargo bench --bench time_warp_bench -- adsr_apply_curve --sample-size 150
