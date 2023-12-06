L1_DIR=l1
LIBL1_DIR=${L1_DIR}/target/release
LIBL1=${LIBL1_DIR}/libl1.a
BUILD_DIR=build
CFLAGS=-Wall -Wextra -Os -g

all: ${BUILD_DIR}/mtetra

${BUILD_DIR}:
	mkdir -p -- "${BUILD_DIR}" "${BUILD_DIR}/inc"

# To simplify things, dependencies are not listed for LIBL1.
# It is a .PHONY target for now so it always gets built.
# cargo checks for dependencies anyway and only creates a new file
# if something was changed, so maybe this is good enough.
${LIBL1}:
	cd -- "${L1_DIR}" && cargo build --release

# TODO: proper dependencies.
# It is .PHONY for now but it is not a great idea.
${BUILD_DIR}/inc/l1.h: | ${BUILD_DIR}
	cbindgen --config "${L1_DIR}/cbindgen.toml" --output "$@" "${L1_DIR}"

${BUILD_DIR}/mtetra: main.c ${BUILD_DIR}/inc/l1.h ${LIBL1} | ${BUILD_DIR}
	${CC} ${CFLAGS} -o "$@" "$<" "-I${BUILD_DIR}/inc" "-L${LIBL1_DIR}" -ll1 -lm

test: ${BUILD_DIR}/mtetra
	$(shell "${BUILD_DIR}/mtetra" | head -c 1000000 > testout.raw)

.PHONY: all test ${LIBL1} ${BUILD_DIR}/inc/l1.h
