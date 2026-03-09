# Dynamic Wallpaper

Dynamic wallpaper is a tool that allows to have automatically changing wallpapers on Wayland supporting graphical environments ;)

## Usage

```
./dynamic_wallpaper [settings file path]
```

## Settings file specification

As settings file structure is very specific it is explained here with artifically created example:
_note: as long as it isn't stated so, the field is mandatory_

```
{
    "config": {
        "default": [string; specifies the default source to be used in case of fetching error],
        "retry_count": [int; how many times plan's source should be fetched before fallback to the default source],
        "refresh_rate": [int; how often should the plan's date be checked; in minutes]
    },
    "plan": [array of objects] [
        {
            "name": [string; name of the plan; not actively used yet],
            "source": [string; file:// or http(s):// source; see "Source resolution" section below for more info],
            "date": [string; cron like format for specifying this plan's appearance date],
            "api": [optional field; used in case source resolves to API that returns an application/json] {
                "source_url_key": [string or array of strings; path to fetch the source image url],
                "filter": [optional] {
                    "field": [string or array of strings; path to string containing constraints],
                    "values": [string or array of strings; contains what values CAN filter's field contain for image to be fetched]
                }
            }
        },
    ]
}
```

### Source resolution

Default source field can contain both file path, directory path or image URL path. In case of directory path (pseudo)random image will be selected and displayed.

In case of URL paths, if header of returned resource contains any of supported http image resource headers it will be displayed. If source's header contains application/json then it will be further processed.

Any other header or incorrect file fallbacks to the default source.

## Supported wallpaper utilities

- hyprpaper
- swww
- mpvpaper
- swaybg
