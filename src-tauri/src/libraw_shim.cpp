#include "libraw/libraw.h"

#include <cstdlib>
#include <cstring>

extern "C" {

struct LapLibRawImage {
  unsigned char *data;
  unsigned int len;
  int format;
  unsigned short width;
  unsigned short height;
  unsigned short colors;
  unsigned short bits;
  int flip;
};

struct LapLibRawMeta {
  // Camera info (imgdata.idata)
  char make[128];
  char model[128];
  char software[128];

  // Capture settings (imgdata.other)
  char artist[64];
  char desc[512];
  long long timestamp;  // time_t
  float iso_speed;
  float shutter;
  float aperture;
  float focal_len;

  // Flash (imgdata.color)
  float flash_used;

  // Lens info (imgdata.lens)
  char lens_make[128];
  char lens_model[128];
  float min_focal;
  float max_focal;
  float max_ap_min_focal;
  float max_ap_max_focal;
};

struct LapLibRawDataErrorState {
  bool occurred;
};

void lap_libraw_data_error_callback(void *callback_data, const char *file,
                                    const long long offset) {
  (void)file;
  (void)offset;
  auto *state = static_cast<LapLibRawDataErrorState *>(callback_data);
  if (state) {
    state->occurred = true;
  }
}

libraw_data_t *lap_libraw_open_buffer(const unsigned char *data, size_t len,
                                      int *err) {
  libraw_data_t *raw = libraw_init(0);
  if (!raw) {
    if (err) {
      *err = -1;
    }
    return nullptr;
  }

  // Use camera white balance and matrix for processed preview rendering.
  // These parameters are consulted during file open for matrix-backed cameras
  // such as many Nikon NEF samples, so they must be set before open_buffer().
  raw->params.use_camera_wb = 1;
  raw->params.use_camera_matrix = 1;

  int ret = libraw_open_buffer(raw, data, len);
  if (ret != LIBRAW_SUCCESS) {
    if (err) {
      *err = ret;
    }
    libraw_close(raw);
    return nullptr;
  }

  if (err) {
    *err = LIBRAW_SUCCESS;
  }
  return raw;
}

void lap_libraw_close(libraw_data_t *raw) {
  if (raw) {
    libraw_close(raw);
  }
}

const char *lap_libraw_strerror(int code) { return libraw_strerror(code); }

int lap_libraw_get_dimensions(libraw_data_t *raw, unsigned int *width,
                              unsigned int *height, int *flip) {
  if (!raw) {
    return LIBRAW_UNSPECIFIED_ERROR;
  }

  int ret = libraw_adjust_sizes_info_only(raw);
  if (ret != LIBRAW_SUCCESS) {
    return ret;
  }

  if (width) {
    *width = static_cast<unsigned int>(raw->sizes.iwidth ? raw->sizes.iwidth
                                                         : raw->sizes.width);
  }
  if (height) {
    *height = static_cast<unsigned int>(raw->sizes.iheight ? raw->sizes.iheight
                                                           : raw->sizes.height);
  }
  if (flip) {
    *flip = raw->sizes.flip;
  }

  return LIBRAW_SUCCESS;
}

int lap_libraw_get_meta(libraw_data_t *raw, LapLibRawMeta *out) {
  if (!raw || !out) {
    return LIBRAW_UNSPECIFIED_ERROR;
  }

  std::memset(out, 0, sizeof(*out));

  // Camera info
  if (raw->idata.make[0]) {
    std::strncpy(out->make, raw->idata.make, sizeof(out->make) - 1);
  }
  if (raw->idata.model[0]) {
    std::strncpy(out->model, raw->idata.model, sizeof(out->model) - 1);
  }
  if (raw->idata.software[0]) {
    std::strncpy(out->software, raw->idata.software, sizeof(out->software) - 1);
  }

  // Capture settings
  if (raw->other.artist[0]) {
    std::strncpy(out->artist, raw->other.artist, sizeof(out->artist) - 1);
  }
  if (raw->other.desc[0]) {
    std::strncpy(out->desc, raw->other.desc, sizeof(out->desc) - 1);
  }
  out->timestamp = raw->other.timestamp;
  out->iso_speed = raw->other.iso_speed;
  out->shutter = raw->other.shutter;
  out->aperture = raw->other.aperture;
  out->focal_len = raw->other.focal_len;

  // Flash
  out->flash_used = raw->color.flash_used;

  // Lens info
  const auto &lens = raw->lens;
  if (lens.LensMake[0]) {
    std::strncpy(out->lens_make, lens.LensMake, sizeof(out->lens_make) - 1);
  }
  if (lens.Lens[0]) {
    std::strncpy(out->lens_model, lens.Lens, sizeof(out->lens_model) - 1);
  } else if (lens.makernotes.Lens[0]) {
    std::strncpy(out->lens_model, lens.makernotes.Lens, sizeof(out->lens_model) - 1);
  }
  out->min_focal = lens.makernotes.MinFocal;
  out->max_focal = lens.makernotes.MaxFocal;
  out->max_ap_min_focal = lens.makernotes.MaxAp4MinFocal;
  out->max_ap_max_focal = lens.makernotes.MaxAp4MaxFocal;

  return LIBRAW_SUCCESS;
}

int lap_libraw_get_thumbnail_count(libraw_data_t *raw) {
  if (!raw) {
    return 0;
  }
  return raw->thumbs_list.thumbcount;
}

int lap_libraw_extract_thumbnail(libraw_data_t *raw, int index,
                                 LapLibRawImage *out) {
  if (!raw || !out) {
    return LIBRAW_UNSPECIFIED_ERROR;
  }

  std::memset(out, 0, sizeof(*out));

  int ret = index >= 0 ? libraw_unpack_thumb_ex(raw, index) : libraw_unpack_thumb(raw);
  if (ret != LIBRAW_SUCCESS) {
    return ret;
  }

  const auto &thumb = raw->thumbnail;
  out->format = static_cast<int>(thumb.tformat);
  out->width = thumb.twidth;
  out->height = thumb.theight;
  out->colors = static_cast<unsigned short>(thumb.tcolors);
  out->bits = thumb.tformat == LIBRAW_THUMBNAIL_BITMAP16 ? 16 : 8;
  out->len = thumb.tlength;
  out->flip = raw->sizes.flip;

  if (index >= 0 && index < raw->thumbs_list.thumbcount) {
    int tflip = raw->thumbs_list.thumblist[index].tflip;
    if (tflip != 0xffff) {
      out->flip = tflip;
    }
  }

  if (thumb.tlength > 0 && thumb.thumb) {
    out->data = static_cast<unsigned char *>(std::malloc(thumb.tlength));
    if (!out->data) {
      return LIBRAW_UNSPECIFIED_ERROR;
    }
    std::memcpy(out->data, thumb.thumb, thumb.tlength);
  }

  return LIBRAW_SUCCESS;
}

int lap_libraw_render_preview(libraw_data_t *raw, int half_size,
                              int strict_data_error, int auto_bright,
                              LapLibRawImage *out) {
  if (!raw || !out) {
    return LIBRAW_UNSPECIFIED_ERROR;
  }

  std::memset(out, 0, sizeof(*out));

  if (half_size) {
    raw->params.half_size = 1;
  }

  LapLibRawDataErrorState data_error = {false};
  if (strict_data_error) {
    libraw_set_dataerror_handler(raw, lap_libraw_data_error_callback,
                                 &data_error);
  }

  int ret = libraw_unpack(raw);
  if (strict_data_error && data_error.occurred) {
    return LIBRAW_IO_ERROR;
  }
  if (ret != LIBRAW_SUCCESS) {
    return ret;
  }

  raw->params.no_auto_bright = auto_bright ? 0 : 1;
  libraw_set_output_bps(raw, 8);

  ret = libraw_dcraw_process(raw);
  if (strict_data_error && data_error.occurred) {
    return LIBRAW_IO_ERROR;
  }
  if (ret != LIBRAW_SUCCESS) {
    return ret;
  }

  int err = LIBRAW_SUCCESS;
  libraw_processed_image_t *image = libraw_dcraw_make_mem_image(raw, &err);
  if (strict_data_error && data_error.occurred) {
    if (image) {
      libraw_dcraw_clear_mem(image);
    }
    return LIBRAW_IO_ERROR;
  }
  if (!image) {
    return err != LIBRAW_SUCCESS ? err : LIBRAW_UNSPECIFIED_ERROR;
  }

  out->format = static_cast<int>(image->type);
  out->width = image->width;
  out->height = image->height;
  out->colors = image->colors;
  out->bits = image->bits;
  out->len = image->data_size;
  out->flip = raw->sizes.flip;

  if (image->data_size > 0) {
    out->data = static_cast<unsigned char *>(std::malloc(image->data_size));
    if (!out->data) {
      libraw_dcraw_clear_mem(image);
      return LIBRAW_UNSPECIFIED_ERROR;
    }
    std::memcpy(out->data, image->data, image->data_size);
  }

  libraw_dcraw_clear_mem(image);
  return LIBRAW_SUCCESS;
}

void lap_libraw_free_buffer(unsigned char *data) {
  if (data) {
    std::free(data);
  }
}

}
