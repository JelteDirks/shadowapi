const express = require('express');
const path = require('path');
const app = express();
const port = 4001;

app.use('/home', express.static(path.join(__dirname, '.')));

app.get('/api', async (req, res) => {
  if (req.headers) {
    process.stderr.write(JSON.stringify(req.headers) + "\n");
  }
  const delay = Math.random();
  await sleep(1000 * delay);
  res.json({ "content": "some content" });
});

app.post('/file', async (req, res) => {
  if (req.headers) {
    process.stderr.write(JSON.stringify(req.headers) + "\n");
  }
  res.send(Buffer.from('This is some binary data', 'utf-8'));
});

app.post('/html', async (req, res) => {
  if (req.headers) {
    process.stderr.write(JSON.stringify(req.headers) + "\n");
  }
  res.send('<html><body><h1>Hello, World!</h1></body></html>');
});

app.post('/json', async (req, res) => {
  if (req.headers) {
    process.stderr.write(JSON.stringify(req.headers) + "\n");
  }
  res.json({ name: 'John Doe', email: 'john.doe@example.com' });
});

app.listen(port, () => {
  console.log(`Server is running on http://localhost:${port}`);
});

function sleep(ms) {
  return new Promise(r => setTimeout(r, ms));
}
