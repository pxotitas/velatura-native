# Catálogo dos scripts auxiliares originais

Fonte: scripts/*.py ativos. Arquivos .new e __pycache__ foram excluídos.

> Cada script precisa de decisão explícita: reimplementar em Rust, integrar por API nativa, ou manter como ferramenta externa opcional. Não manter Python por conveniência sem justificar dependências, segurança e ciclo de vida.

## account.py
- Imports: import json; import os; import pwd; import subprocess; import sys
- Operações detectadas: found = subprocess.run(; done = subprocess.run(; square = subprocess.run(; done = subprocess.run(["pkexec", HELPER, "set"], input=square.stdout,; done = subprocess.run(["pkexec", HELPER, "clear"], capture_output=True, timeout=60)

## applications.py
- Imports: import configparser; import json; import os; import sys
- Operações detectadas: inspecionar fonte integral

## camera.py
- Imports: import fcntl; import importlib.util; import json; import math; import os; import random; import select; import shutil; import signal; import struct; import subprocess; import sys; import time; import zipfile; from pathlib import Path
- Operações detectadas: fd = os.open(path, os.O_RDWR | os.O_NONBLOCK); fd = os.open(path, os.O_RDWR); child = subprocess.Popen([sys.executable, os.path.abspath(__file__), "run", str(fd)],

## capture.py
- Imports: import json; import os; import shutil; import subprocess; import sys; import tempfile; from datetime import datetime; from pathlib import Path
- Operações detectadas: return subprocess.run(command, capture_output=True, text=True,; completed = subprocess.run(["wl-copy", *arguments],; with open(path, "rb") as picture:; subprocess.Popen(

## claude_usage.py
- Imports: import calendar; import json; import os; import subprocess; import sys; import time; from pathlib import Path
- Operações detectadas: with CACHE.open() as handle:; with CACHE.open("w") as handle:; with path.open("r", errors="ignore") as handle:; return subprocess.run(

## clipboard.py
- Imports: import fcntl; import hashlib; import json; import os; import subprocess; import sys; import time; from pathlib import Path
- Operações detectadas: result = subprocess.run(["wl-paste", "--list-types"],; result = subprocess.run(["wl-paste", "--no-newline", "--type", wanted],; with open(store_path(), encoding="utf-8") as handle:; with open(staging, "w", encoding="utf-8") as handle:; handle = open(directory / "clipboard.lock", "a", encoding="utf-8"); with path.open("rb") as body:; subprocess.run(["wl-copy", "--type", entry.get("mime", "text/plain")],; subprocess.run(["wl-copy", "--clear"], timeout=5)

## codex_usage.py
- Imports: import calendar; import json; import os; import sys; import time; from pathlib import Path
- Operações detectadas: with path.open("rb") as handle:

## emoji.py
- Imports: import gettext; import json; import os; import sys; import unicodedata; import xml.etree.ElementTree as ElementTree
- Operações detectadas: with open("/usr/share/iso-codes/json/iso_3166-1.json", encoding="utf-8") as source:; with open(SOURCE, encoding="utf-8") as source:

## fonts.py
- Imports: import json; import shutil; import subprocess; import sys
- Operações detectadas: result = subprocess.run(

## github.py
- Imports: import json; import re; import subprocess; import sys
- Operações detectadas: result = subprocess.run(

## greeting.py
- Imports: import json; import math; import os; import random; import shutil; import subprocess; import sys; import time
- Operações detectadas: with open(PAINT_FILE, encoding="utf-8") as handle:; with open(staging, "w", encoding="utf-8") as handle:; subprocess.run(command, input=stream, capture_output=True, timeout=60, check=True); with open(LOG_FILE, "w", encoding="utf-8") as log:; subprocess.Popen([sys.executable, os.path.abspath(__file__), "draw"],; with open(CHOICE_FILE, encoding="utf-8") as handle:

## lyrics.py
- Imports: import hashlib; import json; import os; import re; import sys; import time; import urllib.error; import urllib.parse; import urllib.request
- Operações detectadas: with urllib.request.urlopen(request, timeout=TIMEOUT) as answer:; with open(path) as file:; with open(staging, "w") as file:

## machine.py
- Imports: from the same sources fastfetch uses so the two agree. Read on demand rather; import json; import os; import pwd; import shutil; import subprocess; import sys
- Operações detectadas: with open(path, encoding="utf-8") as handle:; result = subprocess.run(

## markets.py
- Imports: from Frankfurter: daily reference rates, a month of them, the change since; import json; import subprocess; import sys; from datetime import date, timedelta; from urllib.parse import urlencode
- Operações detectadas: result = subprocess.run(

## monitors.py
- Imports: import json; import subprocess; import sys
- Operações detectadas: result = subprocess.run(

## network.py
- Imports: import json; import shutil; import subprocess; import sys
- Operações detectadas: result = subprocess.run(

## packages.py
- Imports: import json; import os; import re; import shutil; import subprocess; import sys; import threading; import urllib.parse; import urllib.request
- Operações detectadas: return subprocess.run(command, capture_output=True, text=True,; with urllib.request.urlopen(request, timeout=AUR_TIMEOUT) as response:

## privacy.py
- Imports: import ctypes; import glob; import json; import os; import select; import signal; import subprocess; import sys
- Operações detectadas: with open(os.path.join(pid_dir, "cmdline"), "rb") as source:; with open(os.path.join(pid_dir, "comm"), encoding="utf-8") as source:; watch = subprocess.Popen(

## record.py
- Imports: import json; import os; import shutil; import signal; import subprocess; import sys; import time; from datetime import datetime; from pathlib import Path
- Operações detectadas: return subprocess.run(command, capture_output=True, text=True,; started = subprocess.Popen(

## screentime.py
- Imports: import datetime; import fcntl; import json; import os; import re; import select; import signal; import sys; import time
- Operações detectadas: with open(target, "a", encoding="utf-8") as handle:; def open(self, moment):; self.open(now); with open(os.path.join(directory(), f"{day.isoformat()}.tsv"), encoding="utf-8") as handle:; with open(path, encoding="utf-8", errors="replace") as handle:

## servers.py
- Imports: from `ss`, which shows the owning process only for this user's own.; import json; import os; import signal; import subprocess; import sys; import time
- Operações detectadas: result = subprocess.run(; with open("/proc/stat", encoding="ascii") as handle:; with open(f"/proc/{pid}/stat", encoding="ascii", errors="replace") as handle:; with open(f"/proc/{pid}/cmdline", "rb") as handle:

## stats.py
- Imports: import glob; import json; import os; import shutil; import subprocess; import sys; import time
- Operações detectadas: with open(path, encoding="utf-8") as handle:; result = subprocess.run(

## system.py
- Imports: import json; import shutil; import subprocess; import sys
- Operações detectadas: result = subprocess.run(command, capture_output=True, text=True, timeout=5)

## theme_manager.py
- Imports: from its own config directory.; import colorsys; import configparser; import glob; import hashlib; import json; import os; import re; import shutil; import signal; import subprocess; import sys; import textwrap; from urllib.parse import quote; import greeting
- Operações detectadas: with open(os.path.join(XDG_STATE_HOME, "quickshell", "settings.json"),; with open(STATE_FILE, encoding="utf-8") as handle:; with open(STATE_FILE, "w", encoding="utf-8") as handle:; result = subprocess.run(["awww", "query"], capture_output=True, text=True, timeout=5); subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", video, "-frames:v", "1", poster],; subprocess.run(["pkill", *pattern], capture_output=True, timeout=5); subprocess.Popen(["sh", "-c", 'sleep "$1"; shift; exec "$@"', MOTION_TAG,; return subprocess.run(["pgrep", "-x", "mpvpaper"], capture_output=True,; with open(TERMINAL_PALETTE_FILE, "w", encoding="utf-8") as handle:; result = subprocess.run(

## updates.py
- Imports: from here. With no pacman at all the result is unavailable and the module is; import json; import os; import shutil; import subprocess; import sys; import urllib.parse; import urllib.request
- Operações detectadas: return subprocess.run(command, capture_output=True, text=True,; with urllib.request.urlopen(request, timeout=AUR_TIMEOUT) as answer:

## version.py
- Imports: import json; import os; import subprocess; import sys
- Operações detectadas: return subprocess.run(["git", "-C", repo, *args], capture_output=True,

## vms.py
- Imports: import json; import os; import re; import shutil; import signal; import socket; import subprocess; import sys; import time; from typing import NoReturn
- Operações detectadas: with open(path, encoding="utf-8") as source:; with open(temporary, "w", encoding="utf-8") as target:; with open(os.path.join(HOME, name, f"{name}.pid"), encoding="utf-8") as source:; with open(f"/proc/{pid}/cmdline", "rb") as source:; with socket.socket(socket.AF_UNIX) as link:; with open(os.path.join(HOME, name, f"{name}.ports"), encoding="utf-8") as source:; result = subprocess.run(["qemu-img", "info", "-U", "--output=json", disk],; result = subprocess.run(["quickget", "--list-json"], capture_output=True,; with open(CATALOG + ".tmp", "w", encoding="utf-8") as target:; with open(CATALOG, encoding="utf-8") as source:

## weather.py
- Imports: import json; import subprocess; import sys; import urllib.parse
- Operações detectadas: result = subprocess.run(

## Ferramentas e APIs externas para validar

| Ferramenta/API | Domínio | Estratégia nativa/compatibilidade |
|---|---|---|
| niri msg | outputs, event-stream, actions | Implementar IPC/event-stream nativo; não depender de CLI para polling. |
| PipeWire/WirePlumber | áudio, streams, câmera, privacidade | libpipewire / D-Bus apropriado. |
| NetworkManager/nmcli | Wi-Fi e rede | Preferir D-Bus NetworkManager; preservar scan/connect/disconnect/forget. |
| BlueZ/bluetoothctl | Bluetooth | Preferir D-Bus BlueZ; preservar discovery/pair/connect. |
| UPower | bateria | D-Bus UPower; suportar máquina sem bateria. |
| sysfs/DDC/CI/ddcutil | brilho | Cobrir backlight interno, displays externos e hotplug. |
| dcal | calendário | Descobrir protocolo e daemon real; não simular eventos. |
| cava | espectro | Processo opcional com assinantes e shutdown quando não utilizado. |
| wl-paste/wl-copy | clipboard | Preferir APIs Wayland/portal e preservar MIME/imagens. |
| grim/slurp/satty/swappy | captura/seleção/anotação | Auditar probe de ferramentas; não presumir instalação. |
| hyprpicker | picker de cor | Ferramenta opcional; avaliar alternativa Wayland. |
| wlsunset | night light | Lifecycle explícito e detecção de disponibilidade. |
| awww | wallpaper/transições | Preservar daemon, ownership, animated wallpaper e restore. |
| matugen | paleta dinâmica | Preservar extração, geração e sincronização de paletas. |
| pacman/checkupdates/AUR HTTP | pacotes e updates | Ações privilegiadas com confirmação explícita e timeout. |
| ffmpeg | thumbnails/conversão | Opcional, detectar e limitar recursos. |
| systemd-inhibit/logind | sessão/suspensão | Preferir D-Bus logind e liberar inhibitor corretamente. |
| pkexec/PAM/helpers | identidade/autenticação | Segurança crítica; revisão separada e mínimo privilégio. |
| procfs/sysfs/sensors | estatísticas | Preferir leitura direta com parsing testado. |
| APIs HTTP | lyrics/markets/weather/GitHub/usage | Documentar endpoint, schema, cache, timeout, rate limit e privacidade. |

