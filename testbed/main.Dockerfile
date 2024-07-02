FROM node:22-alpine

WORKDIR /usr/app

COPY package*.json ./

RUN npm install --only=production

EXPOSE 4001

COPY main.js ./

CMD ["node", "main.js"]
