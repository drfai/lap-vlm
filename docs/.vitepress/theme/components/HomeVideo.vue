<template>
  <section class="home-video">
    <div class="home-video-inner">
      <h2>Product Demo</h2>
      <p>Watch Lap in action.</p>
      <div class="video-frame">
        <iframe
          src="https://www.youtube-nocookie.com/embed/RbKqNKhbVUs"
          title="Lap demo video"
          loading="lazy"
          allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
          allowfullscreen
        ></iframe>
      </div>
    </div>
  </section>
</template>

<style scoped>
.home-video {
  padding: 24px;
}

.home-video-inner {
  max-width: 1152px;
  margin: 0 auto;
}

.home-video h2 {
  font-size: 28px;
  line-height: 36px;
  margin: 0 0 8px;
}

.home-video p {
  margin: 0 0 16px;
  color: var(--vp-c-text-2);
}

.video-frame {
  position: relative;
  width: 100%;
  padding-top: 56.25%;
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 20px 48px rgba(0, 0, 0, 0.16);
}

.video-frame iframe {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  border: 0;
}

@media (max-width: 768px) {
  .home-video {
    padding: 16px;
  }

  .home-video h2 {
    font-size: 24px;
    line-height: 32px;
  }
}
</style>
