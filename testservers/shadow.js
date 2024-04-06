const express = require('express');
const path = require('path');
const app = express();
const port = 4002;

app.use('/home', express.static(path.join(__dirname, '.')));

app.get('/api', async (req, res) => {
  if (req.headers) {
    process.stderr.write(JSON.stringify(req.headers) + "\n");
  }
  const delay = Math.random();
  await sleep(10 * delay);
  res.json({ "content": "some content" });
});

app.listen(port, () => {
  console.log(`Server is running on http://localhost:${port}`);
});

function sleep(ms) {
  return new Promise(r => setTimeout(r, ms));
}
