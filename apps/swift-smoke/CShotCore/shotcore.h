/* SloerShot shared core - C ABI.
 Maintained to match core/shotcore/src/ffi.rs.
 Link against shotcore.dll (Windows), libshotcore.dylib (macOS), or libshotcore.so (Linux).
 All returned char* are heap-allocated UTF-8 and must be released with shotcore_string_free. */
#ifndef SHOTCORE_H
#define SHOTCORE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SHOTCORE_OK 0
#define SHOTCORE_ERR_ARG -1
#define SHOTCORE_ERR_JSON -3
#define SHOTCORE_ERR_IMAGE -4
#define SHOTCORE_ERR_LICENSE -5
#define SHOTCORE_ERR_EXPIRED -6

/* Returns the core version. Free with shotcore_string_free. */
char *shotcore_version(void);

/* Free a string previously returned by this library. */
void shotcore_string_free(char *ptr);

/* Create a new empty annotation document as JSON. Free with shotcore_string_free. */
char *shotcore_document_new(uint32_t width, uint32_t height);

/* Flatten image_path + doc_json into a composited PNG at out_path. font_path may be NULL. */
int shotcore_export_png(const char *image_path, const char *doc_json, const char *out_path, const char *font_path);

/* Wrap in_path per options_json (BeautifyOptions) and write to out_path. */
int shotcore_beautify_png(const char *in_path, const char *out_path, const char *options_json);

/* Resolve a drag selection against a VirtualDesktop JSON; returns CaptureRegion JSON or NULL. */
char *shotcore_resolve_selection(const char *desktop_json, double x, double y, double w, double h);

/* Search a HistoryStore JSON; returns a JSON array of matching entries or NULL. */
char *shotcore_history_search(const char *store_json, const char *query);
char *shotcore_crop_constrain(double x, double y, double w, double h, uint32_t ratio);
char *shotcore_classify_payload(const char *raw);
char *shotcore_extract_links(const char *text);
char *shotcore_palette_eyedrop(const uint8_t *rgba, uintptr_t len, uint32_t width, uint32_t height, uint32_t x, uint32_t y);
char *shotcore_snap_object(const uint8_t *rgba, uintptr_t len, uint32_t width, uint32_t height, double x, double y, double w, double h, uint8_t tolerance);
char *shotcore_videoedit_output(const char *edit_json, uint64_t source_ms, uint32_t w, uint32_t h, uint32_t channels);
char *shotcore_share_hash_password(const char *plain);
char *shotcore_auto_balance(double cw, double ch, double w, double h);
char *shotcore_combine_stack_vertical(const char *sizes_json, uint32_t gap);
char *shotcore_curved_arrow_path(double fx, double fy, double tx, double ty, double bow, uint32_t segments);
char *shotcore_smart_highlight(const char *ocr_json, double x, double y, double w, double h);
char *shotcore_urlscheme_parse(const char *url);
char *shotcore_print_page_slices(uint32_t content_w, uint32_t content_h, uint32_t page_h, uint32_t overlap);
int shotcore_scale_to_1x_png(const char *in_path, const char *out_path, float scale_factor);
char *shotcore_contrast(const char *hex_a, const char *hex_b);
char *shotcore_guide_markdown(const char *guide_json);
char *shotcore_guide_html(const char *guide_json);
char *shotcore_codeshot_size(const char *card_json, float char_w, float line_h);
char *shotcore_measure(double ax, double ay, double bx, double by);
char *shotcore_auto_zoom(const char *clicks_json, double scale, uint64_t ease_ms, uint64_t hold_ms);
char *shotcore_table_csv(const char *ocr_json, double col_tol);
char *shotcore_table_markdown(const char *ocr_json, double col_tol);
char *shotcore_captions_srt(const char *segments_json);
char *shotcore_captions_vtt(const char *segments_json);
char *shotcore_autotag(const char *text, uint32_t max);
char *shotcore_svg(const char *doc_json);
char *shotcore_align(const char *rects_json, uint32_t edge);
char *shotcore_mockup_size(uint32_t frame, uint32_t content_w, uint32_t content_h);
int shotcore_mask_png(const char *in_path, const char *out_path, uint32_t shape, float radius);

/* Verify a license token against a hex public key at unix time now. Returns a status code. */
int shotcore_license_verify(const char *public_key_hex, const char *token, int64_t now);

/* Detect sensitive data in ocr_json and return doc_json with Redact annotations added. style: 0 blur, 1 pixelate. */
char *shotcore_auto_redact_into(const char *doc_json, const char *ocr_json, uint32_t style, uint32_t strength);

/* Apply a JSON-described fx op (grayscale/sepia/invert/blur/vignette/brightness/contrast/flip/rotate/resize/scale/crop/spotlight/border/jpeg) from in_path to out_path. Returns a status code. */
int shotcore_fx_apply(const char *in_path, const char *out_path, const char *op_json);

/* Opaque stateful annotation editor handle. */
typedef struct ShotEditor ShotEditor;

ShotEditor *shotcore_editor_new(uint32_t width, uint32_t height);
void shotcore_editor_free(ShotEditor *ed);
void shotcore_editor_set_tool(ShotEditor *ed, uint32_t tool);
void shotcore_editor_pointer_down(ShotEditor *ed, double x, double y);
void shotcore_editor_pointer_drag(ShotEditor *ed, double x, double y);
void shotcore_editor_pointer_up(ShotEditor *ed, double x, double y);
int shotcore_editor_undo(ShotEditor *ed);
int shotcore_editor_redo(ShotEditor *ed);
int shotcore_editor_delete_selected(ShotEditor *ed);
int shotcore_editor_bring_to_front(ShotEditor *ed);
int shotcore_editor_send_to_back(ShotEditor *ed);
int shotcore_editor_set_selected_text(ShotEditor *ed, const char *text);
char *shotcore_editor_render_json(ShotEditor *ed);
char *shotcore_editor_document_json(ShotEditor *ed);
int shotcore_editor_can_undo(ShotEditor *ed);
int shotcore_editor_can_redo(ShotEditor *ed);

#ifdef __cplusplus
}
#endif

#endif /* SHOTCORE_H */
