// Pin 7: PUL
// Pin 6: DIR
// Baud: 115200
// Commands: S<rpm>, STOP, START, D0, D1, ?

#define PUL_PIN 7
#define DIR_PIN 6
#define STEPS_PER_REV 400
#define REPORT_INTERVAL_MS 2000

bool running = true;
float currentRPM = 0.1;
unsigned long stepPeriodUs, lastStepTime, lastReportTime;
String inputBuffer = "";

void setRPM(float rpm) {
  if (rpm <= 0 || rpm > 100) return;
  currentRPM = rpm;
  stepPeriodUs = (unsigned long)(60000000.0f / (rpm * STEPS_PER_REV));
}

void printState() {
  Serial.print("STATE:");
  Serial.print(running ? "RUNNING" : "STOPPED");
  Serial.print(" RPM:");
  Serial.print(currentRPM, 3);
  Serial.print(" DIR:");
  Serial.println(digitalRead(DIR_PIN));
}

void handleCommand(String cmd) {
  cmd.trim();
  cmd.toUpperCase();

  if (cmd == "STOP") {
    running = false;
    digitalWrite(PUL_PIN, LOW);
    Serial.println("STOPPED");
  } else if (cmd == "START") {
    running = true;
    lastStepTime = micros();
    Serial.println("RUNNING");
  } else if (cmd == "D0") {
    digitalWrite(DIR_PIN, LOW);
    Serial.println("DIR:CW");
  } else if (cmd == "D1") {
    digitalWrite(DIR_PIN, HIGH);
    Serial.println("DIR:CCW");
  } else if (cmd == "?") {
    printState();
  } else if (cmd.startsWith("S")) {
    float rpm = cmd.substring(1).toFloat();
    if (rpm > 0 && rpm <= 100) {
      setRPM(rpm);
      Serial.print("RPM_SET:");
      Serial.println(rpm, 3);
    } else {
      Serial.println("ERR:RPM must be >=0-100");
    }
  } else {
    Serial.println("ERR:Unknown command");
  }
}

void setup() {
  Serial.begin(115200);
  pinMode(PUL_PIN, OUTPUT);
  pinMode(DIR_PIN, OUTPUT);
  digitalWrite(PUL_PIN, LOW);
  digitalWrite(DIR_PIN, HIGH);
  setRPM(currentRPM);
  Serial.println("READY | Commands: S<rpm>, STOP, START, D0, D1, ?");
  printState();
  lastStepTime = micros();
}

void loop() {
  while (Serial.available()) {
    char c = Serial.read();
    if (c == '\n' || c == '\r') {
      if (inputBuffer.length() > 0) {
        handleCommand(inputBuffer); inputBuffer = "";
      }
    } else {
      inputBuffer += c;
    }
  }

  if (running) {
    unsigned long now = micros();
    if (now - lastStepTime >= stepPeriodUs) {
      digitalWrite(PUL_PIN, HIGH);
      delayMicroseconds(50);
      digitalWrite(PUL_PIN, LOW);
      lastStepTime = now;
    }
  }

  if (millis() - lastReportTime >= REPORT_INTERVAL_MS) {
    if (running) {
      Serial.print("RPM:");
      Serial.println(currentRPM, 1);
    }
    lastReportTime = millis();
  }
}