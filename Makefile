BUILD_DIR     = build
BUILD_INC    := $(BUILD_DIR)/inc
INCLUDE_DIRS += -I$(BUILD_INC)

L1_DIR     = l1
LIBL1_DIR := $(L1_DIR)/target/release
LIBL1     := $(LIBL1_DIR)/libl1.a
L1_H      := $(BUILD_INC)/l1.h

EXECUTABLE   := $(BUILD_DIR)/mtetra
LIBRARIES    := m
DEPENDENCIES :=

OBJECTS := $(BUILD_DIR)/main.o $(LIBL1)

CFLAGS += -MMD -Wall -Wextra -Os -g -std=gnu11 $(INCLUDE_DIRS)

LIBS := \
	$(foreach library, $(LIBRARIES), -l$(library)) \
	$(shell PKG_CONFIG_PATH=$(PKG_CONFIG_PATH) pkg-config --libs $(DEPENDENCIES))


all: $(EXECUTABLE)

test: $(EXECUTABLE)
	$(shell "$(EXECUTABLE)" | head -c 1000000 > testout.raw)

clean:
	rm -rf -- "$(BUILD_DIR)"

$(BUILD_DIR):
	mkdir -p -- "$(BUILD_DIR)"

$(BUILD_INC):
	mkdir -p -- "$(BUILD_INC)"

# cargo checks dependencies itself and only creates a new file
# if something was changed, so this is a .PHONY target.
$(LIBL1):
	cd -- "$(L1_DIR)" && cargo build --release

# Same as above: cbindgen only creates a new file if something was changed.
$(L1_H): | $(BUILD_INC)
	cbindgen --config "$(L1_DIR)/cbindgen.toml" --output "$@" "$(L1_DIR)"

$(BUILD_DIR)/%.o: %.c $(L1_H) | $(BUILD_DIR)
	$(CC) -c $(CFLAGS) "$<" -o "$@"

$(EXECUTABLE): $(OBJECTS) | $(BUILD_DIR)
	$(CC) $(OBJECTS) $(FLAGS) $(LIBS) -o "$@"

.PHONY: all test clean $(LIBL1) $(L1_H)

# Dependencies
-include $(wildcard $(BUILD_DIR)/*.d)
