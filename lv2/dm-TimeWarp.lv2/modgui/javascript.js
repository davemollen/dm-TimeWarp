function(event) {
  function handle_event(symbol, value) {
    
    switch (symbol) {
        case "record":
            const record = event.icon.find("[mod-port-symbol=record]");
            if(value == 1) {
              record.addClass("on");
            } else {
              record.removeClass("on");
            }
            break;
        case "play":
            const play = event.icon.find("[mod-port-symbol=play]");
            if(value == 1) {
              play.addClass("on");
            } else {
              play.removeClass("on");
            }
            break;
        case "flush":
            const flush = event.icon.find("[mod-port-symbol=flush]");
            if(value == 1) {
              flush.addClass("on");
            } else {
              flush.removeClass("on");
            }
            break;
        case "midi_enabled":
          const midi = event.icon.find("[mod-port-symbol=midi_enabled]");
          if(value == 1) {
            midi.addClass("on");
          } else {
            midi.removeClass("on");
          }
          break;
        default:
            break;
    }
  }

  if (event.type == 'start') {
    const ports = event.ports;
    for (const p in ports) {
      handle_event(ports[p].symbol, ports[p].value);
    }
  }
  else if (event.type == 'change') {  
    handle_event(event.symbol, event.value);
  }
}