function(event) {
  function show_correct_time_control_knob(value) {
    const time = event.icon.find("[mod-port-symbol=time]").parent();
    const length = event.icon.find("[mod-port-symbol=length]").parent();
    if(value == 1) {
      time.removeClass("hide")
      length.addClass("hide")
    } else if(value == 2) {
      time.addClass("hide")
      length.removeClass("hide")
    }
  }

  function handle_port_values(symbol, value) {
    switch (symbol) {
      case "record":
        const record = event.icon.find("[mod-port-symbol=record]");
        if(value == 1) {
          record.addClass("on");
        } else {
          record.removeClass("on");
        }
        break;
      case "sample_mode":
        show_correct_time_control_knob(value)
        const sample = event.icon.find("[mod-role=input-parameter]").parent();
        if(value == 3) {
          sample.removeClass("disabled");
          sample.removeClass("prevent-clicks");
        } else {
          sample.addClass("disabled");
          sample.addClass("prevent-clicks");
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
      case "erase":
        const erase = event.icon.find("[mod-port-symbol=erase]");
        if(value == 1) {
          erase.addClass("on");
          event.icon.find("#sample-parameter").text("");
          const sample_mode = event.icon.find(".mod-tab.selected").attr('mod-port-value');
          show_correct_time_control_knob(sample_mode)
        } else {
          erase.removeClass("on");
        }
        break;
      case "midi_enabled": {
        const midi = event.icon.find("[mod-port-symbol=midi_enabled]");
        const attack = event.icon.find("[mod-port-symbol=attack]").parent();
        const decay = event.icon.find("[mod-port-symbol=decay]").parent();
        const sustain = event.icon.find("[mod-port-symbol=sustain]").parent();
        const release = event.icon.find("[mod-port-symbol=release]").parent();
        const sync_position = event.icon.find("[mod-port-symbol=sync_position]").parent();
        const voices = event.icon.find("[mod-port-symbol=voices]").parent();
        const midi_controls = [attack, decay, sustain, release, sync_position, voices];
        if(value == 1) {
          midi.addClass("on");
        } else {
          midi.removeClass("on");
        }
        midi_controls.forEach(function(midi_control) {
          if(value == 1) {
            midi_control.removeClass("disabled");
          } else {
            midi_control.addClass("disabled");
          }
        })
        break;
      }
      case "sync_position":
        const sync_position = event.icon.find("[mod-port-symbol=sync_position]");
        if(value == 1) {
          sync_position.addClass("on");
        } else {
          sync_position.removeClass("on");
        }
        break;
      case "attack":
      case "decay":
      case "release": {
        const element = event.icon.find('[mod-port-symbol=' + symbol + ']').parent();
        if(value >= 10000) {
          element.find('[mod-role="input-control-value"]').text((value / 1000).toFixed(1) + " s");
        } else if(value >= 1000) {
          element.find('[mod-role="input-control-value"]').text((value / 1000).toFixed(2) + " s");
        } else if(value >= 100) {
          element.find('[mod-role="input-control-value"]').text((value).toFixed(0) + " ms");
        } else if(value >= 10) {
          element.find('[mod-role="input-control-value"]').text((value).toFixed(1) + " ms");
        } else {
          element.find('[mod-role="input-control-value"]').text((value).toFixed(2) + " ms");
        }
        break;
      }
      case "dry":
      case "wet": {
        const element = event.icon.find('[mod-port-symbol=' + symbol + ']').parent();
        if(value == -70) {
          element.find('[mod-role="input-control-value"]').text("-inf dB");
        }
        break;
      }
      default:
        break;
    }
  }

  function handle_sample_change(uri, value) {
    const length = event.icon.find("[mod-port-symbol=length]").parent();
    const lengthIsVisibleAlready = !length.hasClass("hide")
    if(lengthIsVisibleAlready) {
      return
    }
    if(uri === 'https://github.com/davemollen/dm-TimeWarp#sample') {
      const time = event.icon.find("[mod-port-symbol=time]").parent();
      const hasLoadedSample = value.length > 0
      if(hasLoadedSample) {
        length.removeClass("hide")
        time.addClass("hide")
      }
    };
  }

  if (event.type == 'start') {
    const ports = event.ports;
    const parameters = event.parameters;
    for (const port in ports) {
      handle_port_values(ports[port].symbol, ports[port].value);
    }
    for (const parameter in parameters) {
      handle_sample_change(parameters[parameter].uri, parameters[parameter].value)
    }
  }
  else if (event.type == 'change') {  
    handle_port_values(event.symbol, event.value);
    handle_sample_change(event.uri, event.value);
  }
}