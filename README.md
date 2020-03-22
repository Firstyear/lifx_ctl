
Lifx Ctl
========

Lifx Ctl is a personal project for controlling lifx bulbs on a layer 2/3 network. It can work
through a router and more.

Each bulb can hold a different "light plan" which is a specified set of behaviours that are able
to change over time. For example, the current system deploys a set of Red Shift plans that lets
your home lights change their whitebalance as the sun sets. We also ship with party hard mode -
turning your house into the nightclub you have always wanted.

Some future goals:

* Make the light configuration ... not hardcoded
* Make light plans configurable (rather than hardcoded)
* Implement the REST front end for App control

To set a colour manually:

    # Blue
    curl -H "Content-Type: application/json" -X POST -d "{\"hue\": 43634, \"sat\": 65535, \"bri\": 47142, \"k\": 3500}" http://127.0.0.1:8081/manual/office
    # Red
    curl -H "Content-Type: application/json" -X POST -d "{\"hue\": 65535, \"sat\": 65535, \"bri\": 65535, \"k\": 3500}" http://127.0.0.1:8081/manual/office

