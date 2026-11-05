cmake_minimum_required(VERSION 3.20)

project(pyc VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

add_library(pyc_core
    src/lexer.cpp
    src/parser.cpp
)
target_include_directories(pyc_core PUBLIC include)

if (MSVC)
    target_compile_options(pyc_core PRIVATE /W4 /permissive-)
else()
    target_compile_options(pyc_core PRIVATE -Wall -Wextra -Wpedantic)
endif()

add_executable(pyc src/main.cpp)
target_link_libraries(pyc PRIVATE pyc_core)

enable_testing()
add_executable(pyc_tests tests/parser_tests.cpp)
target_link_libraries(pyc_tests PRIVATE pyc_core)
add_test(NAME pyc_tests COMMAND pyc_tests)
