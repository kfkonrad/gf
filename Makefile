BINARY  := gf
GOFLAGS := -trimpath

.PHONY: all build test vet fmt check clean install

all: build

build:
	go build $(GOFLAGS) -o $(BINARY) .

test:
	go test ./...

vet:
	go vet ./...

fmt:
	gofmt -l -w .

# check runs static analysis and the test suite
check: vet test

clean:
	rm -f $(BINARY)

install:
	go install $(GOFLAGS) .
