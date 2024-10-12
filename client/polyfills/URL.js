if (typeof URL === "undefined") {
  global.URL = function (url) {
    this.href = url;
    this.toString = function () {
      return this.href;
    };
  };
}
