cmake_minimum_required(VERSION 3.10)

project(render)

include_directories(include)

set(SRC
    src/render.cpp)

set(HEADERS
    include/render.h)

add_library(render ${SRC} ${HEADERS})