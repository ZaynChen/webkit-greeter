local hl = hl

hl.on("hyprland.start", function()
  local log = "/var/log/webkit-greeter/greeter.log"
  local old = "/var/log/webkit-greeter/greeter.old.log"
  local cmd1 = "mv " .. log .. " " .. old
  local cmd2 = "webkit-greeter 2>" .. log
  local cmd3 = "hyprctl dispatch exit"
  hl.exec_cmd(cmd1 .. ";" .. cmd2 .. ";" .. cmd3)
end)

--  preferred - use the display’s preferred size and refresh rate.
--  auto - let Hyprland decide on a position.
--    By default, it places each new monitor to the right of existing ones,
--    using the monitor’s top left corner as the root point.
hl.monitor({
  output   = "",
  mode     = "preferred",
  position = "auto",
  scale    = "1",
})

hl.config({
  general = {
    border_size = 0,
    gaps_in = 0,
    gaps_out = 0,
    locale = "zh_CN.UTF-8"
  },
  decoration = { blur = { enabled = false }, shadow = { enabled = false } },
  animations = { enabled = false },
  misc = {
    disable_hyprland_logo = true,
    disable_splash_rendering = true,
    focus_on_activate = true,
  },
  input = { natural_scroll = true }
})
