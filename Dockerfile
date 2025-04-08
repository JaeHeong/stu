# Stage 1: Backend 빌드 (Gradle & Java 21 사용) - 변경 없음
FROM gradle:jdk21 AS backend-builder
WORKDIR /app/backend
COPY backend/build.gradle.kts backend/pom.xml ./
COPY backend/src ./src
RUN gradle build --no-daemon -x test

# Stage 2: Frontend 빌드 (Node.js 사용) - 변경 없음
FROM node:20 AS frontend-builder
WORKDIR /app/front
COPY front/package.json front/package-lock.json ./
RUN npm install
COPY front/ ./
RUN npm run build

# Stage 3: 최종 이미지 생성 (Ubuntu 기반)
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
# --- apt 캐시 정리 후 업데이트 및 설치 ---
RUN rm -rf /var/lib/apt/lists/* \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
    openssh-server \
    openjdk-21-jre-headless \
    supervisor \
    nginx \
    procps \
    curl \
    sed \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* 

RUN mkdir -p /var/run/sshd
RUN echo "root:password" | chpasswd
RUN sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config \
    && sed -i 's/#PasswordAuthentication yes/PasswordAuthentication yes/' /etc/ssh/sshd_config \
    && sed -i 's/PasswordAuthentication no/PasswordAuthentication yes/' /etc/ssh/sshd_config \
    && sed -i 's/#ListenAddress 0.0.0.0/ListenAddress 0.0.0.0/' /etc/ssh/sshd_config \
    && sed -i 's/#ListenAddress ::/ListenAddress ::/' /etc/ssh/sshd_config \
    && echo "AcceptEnv LANG LC_*" >> /etc/ssh/sshd_config

WORKDIR /app
COPY --from=backend-builder /app/backend/build/libs/*.jar ./backend.jar
COPY --from=frontend-builder /app/front/build /var/www/html/front
COPY nginx.conf /etc/nginx/sites-available/default
RUN ln -sf /dev/stdout /var/log/nginx/access.log \
    && ln -sf /dev/stderr /var/log/nginx/error.log

COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

EXPOSE 8081

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]