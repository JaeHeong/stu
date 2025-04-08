# Web Terminal for S3 (using stu)

이 프로젝트는 Rust 기반의 TUI 도구인 **[stu](https://github.com/lusingander/stu)**를 사용하여 AWS S3 버킷과 상호작용할 수 있는 **웹 기반 터미널 인터페이스**를 제공합니다.  
모든 구성 요소(`stu`, 백엔드, 프론트엔드, SSH 서버)는 **단일 Docker 컨테이너** 내에서 실행됩니다.

---

## 🚀 Prerequisites

- [Docker](https://www.docker.com/get-started) 설치 및 실행
- AWS Access Key ID와 Secret Access Key

---

## 🛠️ Build

루트 디렉토리(예: `Dockerfile-stu`가 있는 위치)에서 다음 명령어를 실행하여 Docker 이미지를 빌드하세요:

```bash
docker build -t web-terminal . -f Dockerfile-stu
▶️ Run
컨테이너를 실행할 때, AWS 자격 증명과 리전을 환경 변수로 전달해야 합니다.
아래 명령어에서 YOUR_... 값을 실제 값으로 바꿔 실행하세요:

bash
복사
편집
docker run -d \
  -p 8081:8081 \
  --name web-terminal \
  -e AWS_ACCESS_KEY_ID="YOUR_ACCESS_KEY_ID" \
  -e AWS_SECRET_ACCESS_KEY="YOUR_SECRET_ACCESS_KEY" \
  -e AWS_REGION="ap-northeast-2" \
  # Optional: 특정 AWS 프로파일 사용 시 아래 주석 해제
  # -e AWS_PROFILE="your-profile-name" \
  web-terminal
🌐 환경 변수 설명
환경 변수	설명
AWS_ACCESS_KEY_ID	사용할 AWS Access Key ID (필수)
AWS_SECRET_ACCESS_KEY	사용할 AWS Secret Access Key (필수)
AWS_REGION	대상 S3 버킷의 AWS 리전 (예: ap-northeast-2, us-east-1) (필수)
AWS_PROFILE	(선택 사항) 로컬 ~/.aws/config의 프로파일 이름 지정. 여전히 자격 증명은 다른 방식(환경 변수 등)으로 필요할 수 있음
⚠️ 중요: 실제 배포 전에는 반드시 "YOUR_ACCESS_KEY_ID" 및 "YOUR_SECRET_ACCESS_KEY" 값을 안전한 실제 자격 증명으로 교체하세요.
이 정보는 절대 공개 저장소에 포함되지 않도록 주의하세요.

🌍 웹 터미널 접속
컨테이너가 성공적으로 실행되면, 웹 브라우저에서 다음 주소로 접속합니다:

arduino
복사
편집
http://localhost:8081
브라우저에 stu 기반의 S3 터미널 인터페이스가 표시됩니다. 실행 시 전달한 AWS 자격 증명을 기반으로 S3 버킷과 상호작용할 수 있습니다.

📎 참고
stu는 터미널 기반 S3 클라이언트입니다.

Docker 이미지에는 stu, 웹 UI, SSH 서버, 연결 핸들러 등이 포함되어 있습니다.

보안을 위해 자격 증명은 .env 파일을 사용하거나 AWS Vault, IAM Role, ECS Task Role 등으로 관리하는 것이 좋습니다.
```
