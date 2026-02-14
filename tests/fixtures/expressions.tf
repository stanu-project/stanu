locals {
  a = 1 + 2 * 3
  b = var.enabled ? "yes" : "no"
  c = length(var.list)
  d = [for s in var.list : upper(s) if s != ""]
  e = {for k, v in var.map : k => upper(v)}
  f = var.items[*].name
  g = -5
  h = !var.flag
  i = (1 + 2) * 3
  j = [1, 2, 3]
  k = {a = 1, b = 2}
  l = "hello ${var.name} world"
}
