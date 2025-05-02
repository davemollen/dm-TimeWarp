function (event, funcs)
{
    if (event.type == 'start')
    {
        switch (event.symbol) {
            case 'time':
                console.log('time', event.value)
                return
        }
        for (var i in event.parameters)
        {
            if (event.parameters[i].uri === 'https://github.com/davemollen/dm-TimeWarp#sample')
            {
                console.log('sample', event.parameters[i].value);
                break;
            }
        }
    }
    else if (event.type == 'change')
    {
        switch (event.symbol) {
            case 'time':
                console.log('time', event.value)
                return
        }
        if (event.uri === 'https://github.com/davemollen/dm-TimeWarp#sample')
        {
            console.log('sample', event.value);
        }
    }
}
