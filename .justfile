user    := "atareao"
name    := `basename ${PWD}`
version := `git tag -l  | tail -n1`

default:
    @just --list

build:
    echo {{version}}
    echo {{name}}
    docker build -t {{user}}/{{name}}:{{version}} \
                 -t {{user}}/{{name}}:latest \
                 .

tag:
    docker tag {{user}}/{{name}}:{{version}} {{user}}/{{name}}:latest

push:
    docker push {{user}}/{{name}} --all-tags

run:
    docker run --rm \
               --init \
               -e RUST_LOG='debug' \
               -e PORT='8080' \
               -v ${PWD}:/app/content \
               -v ${PWD}:/app/ssets \
               --publish '8080:8080' \
               --name {{name}} \
               {{user}}/{{name}}

exe:
    docker run --rm \
               --init \
               -it \
               -e RUST_LOG='debug' \
               -e PORT='8080' \
               -v ${PWD}:/app/content \
               -v ${PWD}:/app/ssets \
               --publish '8080:8080' \
               --name {{name}} \
               {{user}}/{{name}} \
               sh

test:
    echo {{version}}
    echo {{name}}
    docker build -t {{user}}/{{name}}:test \
                 .
    docker push {{user}}/{{name}}:test

