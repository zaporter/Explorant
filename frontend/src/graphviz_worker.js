addEventListener("message", event => {
  const [d3, gviz, dotSrc] = event.data;

  gviz.transition(function() {
    return d3.transition()
      .delay(0)
      .duration(100);
  }).renderDot(dotSrc.dot)//.on("end", interactive);
});
