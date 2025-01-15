#*******************************************************************************
#  Ledger App
#  (c) 2018 - 2024 Zondax AG
#  (c) 2017 Ledger
#
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
#*******************************************************************************

ifeq ($(BOLOS_SDK),)
$(error BOLOS_SDK is not set)
endif

MY_DIR := $(dir $(lastword $(MAKEFILE_LIST)))

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.installer_script

include $(BOLOS_SDK)/Makefile.defines

# Set the default value for PRODUCTION_BUILD to 0 if not already defined
PRODUCTION_BUILD ?= 1

$(info ************ PENUMBRA_NAME  = [$(TARGET_NAME)])

# Display whether this is a production build or for internal use
ifeq ($(PRODUCTION_BUILD), 1)
    $(info ************ PRODUCTION_BUILD  = [PRODUCTION BUILD])
else
    $(info ************ PRODUCTION_BUILD  = [INTERNAL USE])
endif

# Display whether swap functionality is enabled or not
ifdef HAVE_SWAP
    $(info ************ HAVE_SWAP  = [ENABLED])
    DEFINES += HAVE_SWAP=$(HAVE_SWAP)
else
    $(info ************ HAVE_SWAP  = [DISABLED])
endif

# Add the PRODUCTION_BUILD definition to the compiler flags
DEFINES += PRODUCTION_BUILD=$(PRODUCTION_BUILD)

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.app_testing

ifndef COIN
COIN=UM
endif

include $(CURDIR)/Makefile.version

$(info COIN  = [$(COIN)])

ifeq ($(COIN), UM)
# Main app configuration
DEFINES += APP_STANDARD
APPNAME = "Penumbra"
APPPATH = "44'/6532'"

else
define error_message

COIN value not supported: [$(COIN)]

endef
$(error "$(error_message)")
endif

$(info PATHS LIST = $(APPPATH))

APP_LOAD_PARAMS = --curve secp256k1 $(COMMON_LOAD_PARAMS) --path $(APPPATH)
INCLUDES_PATH += $(CURDIR)/src/protobuf

ifeq ($(TARGET_NAME),TARGET_NANOS)
	APP_STACK_MIN_SIZE := 1600
else
	APP_STACK_MIN_SIZE := 1752
endif

# Flag to compile all Penumbra actions
FULL_APP ?= 0
ifeq ($(FULL_APP),1)
DEFINES += FULL_APP
$(info ************ FULL APP ENABLED ************)
endif

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.devices

$(info TARGET_NAME  = [$(TARGET_NAME)])
$(info ICONNAME  = [$(ICONNAME)])

ifndef ICONNAME
$(error ICONNAME is not set)
endif

ifeq ($(TARGET_NAME),TARGET_NANOS)
RUST_TARGET:=thumbv6m-none-eabi
endif

ifeq ($(TARGET_NAME),TARGET_NANOX)
RUST_TARGET:=thumbv6m-none-eabi
endif

ifeq ($(TARGET_NAME),TARGET_NANOS2)
#RUST_TARGET:=thumbv8m.main-none-eabi
RUST_TARGET:=thumbv6m-none-eabi
endif

ifeq ($(TARGET_NAME),TARGET_STAX)
RUST_TARGET:=thumbv6m-none-eabi
endif

ifeq ($(TARGET_NAME),TARGET_FLEX)
RUST_TARGET:=thumbv6m-none-eabi
endif

$(info ************ RUST_TARGET  = [$(RUST_TARGET)])

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.platform
APP_SOURCE_PATH += $(CURDIR)/rust/include
CFLAGS += -Wvla
LDFLAGS += -z muldefs
LDLIBS   += -lm -lgcc -lc
LDLIBS += -L$(MY_DIR)rust/target/$(RUST_TARGET)/release -lrslib


ifeq ($(DEBUG), 1)
CFLAGS += -O3 -Os -Wno-unknown-pragmas -Wno-unused-parameter -g
else
CFLAGS += -O3 -Os -Wno-unknown-pragmas -Wno-unused-parameter
endif

APP_CUSTOM_LINK_DEPENDENCIES = rust

.PHONY: rust
rust:
ifeq ($(FULL_APP), 1)
	cd rust && RUSTC_BOOTSTRAP=1 CARGO_HOME="$(CURDIR)/rust/.cargo" cargo build --target $(RUST_TARGET) --release --features full_app
else
	cd rust && RUSTC_BOOTSTRAP=1 CARGO_HOME="$(CURDIR)/rust/.cargo" cargo build --target $(RUST_TARGET) --release
endif

# Before linking, we need to be sure rust lib is there
bin/app.elf: rust

.PHONY: rust_clean
rust_clean:
	cd rust && CARGO_HOME="$(CURDIR)/rust/.cargo" cargo clean

clean: rust_clean

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.side_loading

# Import generic rules from the SDK
include $(BOLOS_SDK)/Makefile.rules

#add dependency on custom makefile filename
dep/%.d: %.c Makefile

.PHONY: listvariants
listvariants:
	@echo VARIANTS COIN UM

.PHONY: version
version:
	@echo "v$(APPVERSION)" > app.version
