CARGO = cargo
TARGET = target/release/clipsearch
INSTALL_DIR = ~/.local/bin
ASSETS_DIR = assets
APP_NAME = clipsearch
DESKTOP_FILE_TEMPLATE = ${ASSETS_DIR}/${APP_NAME}.desktop.template
DESKTOP_FILE = ${ASSETS_DIR}/${APP_NAME}.desktop
LOCAL_DESKTOP_DIR = ~/.local/share/applications
ICON_PNG_256 = ${ASSETS_DIR}/clipsearch.png
LOCAL_ICON_DIR_256 = ~/.local/share/icons/hicolor/256x256/apps
VERSION=$(shell grep '^version =' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

.PHONY: all clean install uninstall

# Default target
all: release

# Compile the project in release mode
release:
	$(CARGO) build --release

# Clean the project
clean:
	@$(CARGO) clean --quiet
	@echo "Cleaned the rust project"
	@rm -f $(DESKTOP_FILE)
	@echo "Cleaned $(DESKTOP_FILE)"

# Generate the .desktop file from template with dynamic version
$(DESKTOP_FILE): $(DESKTOP_FILE_TEMPLATE)
	sed 's/{{VERSION}}/$(VERSION)/g' $(DESKTOP_FILE_TEMPLATE) > $(DESKTOP_FILE)

install: release $(DESKTOP_FILE)
	install -Dm755 $(TARGET) $(INSTALL_DIR)/$(APP_NAME)
	@echo "Installed $(APP_NAME) to $(INSTALL_DIR)"
	install -Dm644 $(DESKTOP_FILE) $(LOCAL_DESKTOP_DIR)/$(DESKTOP_FILE)
	@echo "Desktop entry created at $(LOCAL_DESKTOP_DIR)/$(DESKTOP_FILE)"
	install -Dm644 $(ICON_PNG_256) $(LOCAL_ICON_DIR_256)/$(APP_NAME).png
	@echo "Installed icon to $(LOCAL_ICON_DIR_256)/$(APP_NAME).png"

uninstall:
	rm -f $(INSTALL_DIR)/$(APP_NAME)
	@echo "Removed $(APP_NAME) from $(INSTALL_DIR)"
	rm -f $(LOCAL_DESKTOP_DIR)/$(DESKTOP_FILE)
	@echo "Removed desktop entry from $(LOCAL_DESKTOP_DIR)"
	rm -f $(LOCAL_ICON_DIR_256)/$(APP_NAME).png
	@echo "Removed icon from $(LOCAL_ICON_DIR_256)"
