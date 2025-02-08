package main

import "core:math/rand"
import "core:fmt"


main :: proc(){
    fmt.println(rand.float64_von_mises(10, 1.5))
}