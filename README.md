# IngenSkud

A top-down game, in which you combine different elements and spelltypes to create spells to kill opponents. These different elements interact with parts of the terrain/map in different ways.
Such as using certain ice spells on water, will make it freeze and root enemies already walking in the water.

## Build requirements

The below are adaptations of the documentation on `ggez`. Go there for more (or less) information.

## Windows

Should just be able to compile. MSVC toolchain works best.

## Linux

### Debian

The following packages are required:

```sh
apt install libasound2-dev libudev-dev pkg-config
```

### Redhat

Same libraries as Debian, slightly different names. On CentOS 7, at
least you can install them with:

```sh
yum install alsa-lib-devel
```
