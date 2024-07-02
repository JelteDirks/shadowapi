FROM node:22-alpine

WORKDIR /usr/app

COPY package*.json ./

RUN npm install --only=production

EXPOSE 4002

COPY shadow.js ./

CMD ["node", "shadow.js"]
