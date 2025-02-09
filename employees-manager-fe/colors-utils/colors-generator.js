const tinycolor = require("tinycolor2");
const fs = require("fs");

const buildColor = (prefix, value, name) => ({
  varName: "$" + prefix + name,
  varValue: tinycolor(value).toHexString(),
});
const mul = (rgb1, rgb2) => {
  rgb1.b = Math.floor((rgb1.b * rgb2.b) / 255);
  rgb1.g = Math.floor((rgb1.g * rgb2.g) / 255);
  rgb1.r = Math.floor((rgb1.r * rgb2.r) / 255);
  return tinycolor("rgb " + rgb1.r + " " + rgb1.g + " " + rgb1.b);
};

const generateColorPalette = (prefix, colorHex) => {
  var baseLight = tinycolor("#ffffff");
  var baseDark = mul(tinycolor(colorHex).toRgb(), tinycolor(colorHex).toRgb());
  var baseTriad = tinycolor(colorHex).tetrad();
  return [
    buildColor(prefix, tinycolor.mix(baseLight, colorHex, 12), "50"),
    buildColor(prefix, tinycolor.mix(baseLight, colorHex, 30), "100"),
    buildColor(prefix, tinycolor.mix(baseLight, colorHex, 50), "200"),
    buildColor(prefix, tinycolor.mix(baseLight, colorHex, 70), "300"),
    buildColor(prefix, tinycolor.mix(baseLight, colorHex, 85), "400"),
    buildColor(prefix, tinycolor.mix(baseLight, colorHex, 100), "500"),
    buildColor(prefix, tinycolor.mix(baseDark, colorHex, 87), "600"),
    buildColor(prefix, tinycolor.mix(baseDark, colorHex, 70), "700"),
    buildColor(prefix, tinycolor.mix(baseDark, colorHex, 54), "800"),
    buildColor(prefix, tinycolor.mix(baseDark, colorHex, 25), "900"),
    buildColor(
      prefix,
      tinycolor.mix(baseDark, baseTriad[4], 15).saturate(80).lighten(65),
      "A100"
    ),
    buildColor(
      prefix,
      tinycolor.mix(baseDark, baseTriad[4], 15).saturate(80).lighten(55),
      "A200"
    ),
    buildColor(
      prefix,
      tinycolor.mix(baseDark, baseTriad[4], 15).saturate(100).lighten(45),
      "A400"
    ),
    buildColor(
      prefix,
      tinycolor.mix(baseDark, baseTriad[4], 15).saturate(100).lighten(40),
      "A700"
    ),
  ];
};

class SassColorGenerator {
  constructor() {
    this.colors = [];
  }
  addColor(prefix, colorHex) {
    this.colors.push(...generateColorPalette(prefix, colorHex));
  }
  generateFile() {
    let text = "";
    this.colors.forEach((item, index) => {
      text += `${item.varName}: ${item.varValue};\n${
        index % 14 === 13 ? "\n" : ""
      }`;
    });

    fs.writeFileSync("./colors-utils/colors.scss", text, "utf-8");
    console.log("File generated to colors-utils/colors.scss");
  }
}

const scg = new SassColorGenerator();
scg.addColor("green", "#28a238");
scg.addColor("gray", "#636465");
scg.addColor("lightblue", "#4ebdc3");
scg.addColor("black", "#1e1e1c");
scg.addColor("blue", "#717e8f");
scg.addColor("red", "#e84e0f");
scg.addColor("yellow", "#fbba00");
scg.generateFile();
