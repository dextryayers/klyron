import polka from 'polka'
polka().get('/', (req, res) => { res.end('Hello Polka') }).listen(3000)
