BUILD_DIR      = build
BUILD_INC_DIR := $(BUILD_DIR)/inc
BUILD_L2_DIR  := $(BUILD_DIR)/l2

L1_DIR     = l1
LIBL1_DIR := $(L1_DIR)/target/release
LIBL1     := $(LIBL1_DIR)/libl1.a
L1_H      := $(BUILD_INC_DIR)/l1.h

EXECUTABLE   := $(BUILD_DIR)/mtetra
LIBRARIES    := m SoapySDR
DEPENDENCIES :=

INCLUDE_DIRS += -I$(BUILD_INC_DIR) -Il2

OBJECTS := $(BUILD_DIR)/main.o $(BUILD_L2_DIR)/l2.o $(LIBL1)

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
	mkdir -p -- "$@"

$(BUILD_INC_DIR):
	mkdir -p -- "$@"

$(BUILD_L2_DIR):
	mkdir -p -- "$@"

# cargo checks dependencies itself and only creates a new file
# if something was changed, so this is a .PHONY target.
$(LIBL1):
	cd -- "$(L1_DIR)" && cargo build --release

# Same as above: cbindgen only creates a new file if something was changed.
$(L1_H): | $(BUILD_INC_DIR)
	cbindgen --config "$(L1_DIR)/cbindgen.toml" --output "$@" "$(L1_DIR)"

$(BUILD_DIR)/%.o: %.c $(L1_H) | $(BUILD_DIR)
	$(CC) -c $(CFLAGS) "$<" -o "$@"

$(BUILD_L2_DIR)/%.o: l2/%.c $(L1_H) | $(BUILD_L2_DIR)
	$(CC) -c $(CFLAGS) "$<" -o "$@"

$(EXECUTABLE): $(OBJECTS) | $(BUILD_DIR)
	$(CC) $(OBJECTS) $(FLAGS) $(LIBS) -o "$@"

.PHONY: all test clean $(LIBL1) $(L1_H)

# Dependencies
-include $(wildcard $(BUILD_DIR)/*.d $(BUILD_L2_DIR)/*.d)
