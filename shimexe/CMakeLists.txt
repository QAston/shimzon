cmake_minimum_required (VERSION 3.18)

project (shim VERSION 0.1
    DESCRIPTION "shim.exe"
    LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(shim shim.cpp)
install(TARGETS shim DESTINATION .)