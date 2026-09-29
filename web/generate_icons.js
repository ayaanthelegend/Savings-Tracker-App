import fs from "fs";
import path from "path";
import zlib from "zlib";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// CRC32 table & function for PNG chunks
const crcTable = new Uint32Array(256);
for (let n = 0; n < 256; n++) {
  let c = n;
  for (let k = 0; k < 8; k++) {
    c = (c & 1) ? (0xedb88320 ^ (c >>> 1)) : (c >>> 1);
  }
  crcTable[n] = c;
}

function crc32(buf) {
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc = crcTable[(crc ^ buf[i]) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function makeChunk(type, data) {
  const len = data.length;
  const chunk = Buffer.alloc(12 + len);
  chunk.writeUInt32BE(len, 0);
  chunk.write(type, 4, 4, "ascii");
  data.copy(chunk, 8);
  const typeAndData = chunk.subarray(4, 8 + len);
  const crc = crc32(typeAndData);
  chunk.writeUInt32BE(crc, 8 + len);
  return chunk;
}

function generatePng(width, height) {
  // RGBA buffer: (width * 4 + 1) per row (filter byte 0)
  const rowLen = width * 4 + 1;
  const rawData = Buffer.alloc(rowLen * height);

  // Background color #11141A (17, 20, 26)
  // Inner ring #6C7CF0 (108, 124, 240)
  // Coin/Accent #3FCDA8 (63, 205, 168) & #E8B93F (232, 185, 63)
  const cx = width / 2;
  const cy = height / 2;
  const radius = width * 0.42;
  const innerR = width * 0.35;
  const cornerR = width * 0.22;

  for (let y = 0; y < height; y++) {
    const rowOffset = y * rowLen;
    rawData[rowOffset] = 0; // Filter None

    for (let x = 0; x < width; x++) {
      const pxOffset = rowOffset + 1 + x * 4;
      
      // Rounded rect background
      const dx = Math.max(Math.abs(x - cx) - (cx - cornerR), 0);
      const dy = Math.max(Math.abs(y - cy) - (cy - cornerR), 0);
      const distFromCorner = Math.sqrt(dx * dx + dy * dy);

      if (distFromCorner > cornerR) {
        // Outside rounded rect (transparent or dark bg)
        rawData[pxOffset] = 17;
        rawData[pxOffset + 1] = 20;
        rawData[pxOffset + 2] = 26;
        rawData[pxOffset + 3] = 0;
        continue;
      }

      // Default dark theme background #11141A
      let r = 17, g = 20, b = 26, a = 255;

      const dCenter = Math.sqrt((x - cx) * (x - cx) + (y - cy) * (y - cy));

      // Circular accent ring
      if (dCenter <= radius && dCenter >= radius - width * 0.04) {
        r = 108; g = 124; b = 240; // #6C7CF0
      }
      // Inner circle glow
      else if (dCenter < radius - width * 0.04 && dCenter > innerR) {
        r = 23; g = 27; b = 35; // #171B23
      }
      // Central icon: stylized wallet / card / coin
      else if (dCenter <= innerR) {
        // Card shape
        const cardW = width * 0.46;
        const cardH = height * 0.28;
        const inCardX = Math.abs(x - cx) <= cardW / 2;
        const inCardY = Math.abs(y - (cy + height * 0.04)) <= cardH / 2;

        if (inCardX && inCardY) {
          // Card body #1C212B
          r = 28; g = 33; b = 43;
          // Top stripe of card
          if (y - (cy + height * 0.04) <= -cardH / 2 + height * 0.07) {
            r = 108; g = 124; b = 240; // #6C7CF0
          }
          // Chip / coin on card
          const chipX = cx - cardW * 0.25;
          const chipY = cy + height * 0.04;
          if (Math.abs(x - chipX) <= width * 0.06 && Math.abs(y - chipY) <= height * 0.05) {
            r = 232; g = 185; b = 63; // #E8B93F
          }
        }

        // Coin floating above card
        const coinCx = cx + width * 0.12;
        const coinCy = cy - height * 0.14;
        const coinR = width * 0.14;
        const dCoin = Math.sqrt((x - coinCx) * (x - coinCx) + (y - coinCy) * (y - coinCy));

        if (dCoin <= coinR) {
          if (dCoin >= coinR - width * 0.02) {
            r = 232; g = 185; b = 63; // #E8B93F border
          } else {
            r = 63; g = 205; b = 168; // #3FCDA8
          }
        }
      }

      rawData[pxOffset] = r;
      rawData[pxOffset + 1] = g;
      rawData[pxOffset + 2] = b;
      rawData[pxOffset + 3] = a;
    }
  }

  // PNG Header
  const signature = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

  // IHDR Chunk
  const ihdrData = Buffer.alloc(13);
  ihdrData.writeUInt32BE(width, 0);
  ihdrData.writeUInt32BE(height, 4);
  ihdrData[8] = 8; // 8-bit depth
  ihdrData[9] = 6; // RGBA
  ihdrData[10] = 0; // Deflate
  ihdrData[11] = 0; // Standard filter
  ihdrData[12] = 0; // No interlace
  const ihdrChunk = makeChunk("IHDR", ihdrData);

  // IDAT Chunk
  const compressed = zlib.deflateSync(rawData);
  const idatChunk = makeChunk("IDAT", compressed);

  // IEND Chunk
  const iendChunk = makeChunk("IEND", Buffer.alloc(0));

  return Buffer.concat([signature, ihdrChunk, idatChunk, iendChunk]);
}

const iconsDir = path.join(__dirname, "public", "icons");
fs.mkdirSync(iconsDir, { recursive: true });

// 1. apple-touch-icon.png (180x180)
const appleIcon = generatePng(180, 180);
fs.writeFileSync(path.join(iconsDir, "apple-touch-icon.png"), appleIcon);
fs.writeFileSync(path.join(__dirname, "public", "apple-touch-icon.png"), appleIcon);

// 2. icon-192.png
const icon192 = generatePng(192, 192);
fs.writeFileSync(path.join(iconsDir, "icon-192.png"), icon192);

// 3. icon-512.png
const icon512 = generatePng(512, 512);
fs.writeFileSync(path.join(iconsDir, "icon-512.png"), icon512);

// 4. SVG vector icon
const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
  <rect width="512" height="512" rx="112" fill="#11141A"/>
  <circle cx="256" cy="256" r="215" fill="none" stroke="#6C7CF0" stroke-width="18" stroke-dasharray="12 12"/>
  <rect x="136" y="210" width="240" height="150" rx="20" fill="#171B23" stroke="#242938" stroke-width="8"/>
  <rect x="136" y="240" width="240" height="32" fill="#1C212B"/>
  <rect x="160" y="300" width="48" height="36" rx="6" fill="#E8B93F"/>
  <circle cx="330" cy="180" r="64" fill="#3FCDA8" stroke="#11141A" stroke-width="8"/>
  <text x="330" y="202" font-family="Space Grotesk, sans-serif" font-size="64" font-weight="bold" fill="#11141A" text-anchor="middle">₨</text>
</svg>`;
fs.writeFileSync(path.join(iconsDir, "icon.svg"), svg);
fs.writeFileSync(path.join(__dirname, "public", "favicon.svg"), svg);

console.log("Generated PWA icons successfully!");
