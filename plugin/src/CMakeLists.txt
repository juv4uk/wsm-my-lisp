add_library(wsm-my-lisp-cyberpunk-plugin SHARED)

target_sources(
  wsm-my-lisp-cyberpunk-plugin
  PRIVATE
    Main.cpp
)

target_include_directories(
  wsm-my-lisp-cyberpunk-plugin
  PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}"
)

target_link_libraries(
  wsm-my-lisp-cyberpunk-plugin
  PRIVATE
    RED4ext::SDK
)

set_target_properties(
  wsm-my-lisp-cyberpunk-plugin
  PROPERTIES
    OUTPUT_NAME "wsm-my-lisp-cyberpunk-plugin"
)
