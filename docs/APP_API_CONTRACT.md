# Contrato público Backend ↔ Codeína

**Versão:** 0.1-draft<br>
**Estado:** draft preliminar; revisão técnica concluiu que **NÃO ESTÁ PRONTO** para servir de base à portabilidade 1:1. Ver `docs/APP_API_REVIEW.md` para achados, gravidade e requisitos bloqueadores.<br>
**Escopo:** semântica de estado, eventos, comandos, disponibilidade e erros. Não escolhe Relm4, Tokio, canal concreto ou composição GTK.

Este documento complementa ARCHITECTURE.md, IPC_AUDIT.md, BACKEND_AUDIT.md e INTEGRATION_AUDIT.md. É uma API interna Rust; não exige D-Bus nem socket externo. A CLI legada será um adaptador sobre o mesmo domínio, preservando targets/verbs e saída compatível.

## 1. Princípios obrigatórios

1. Snapshot é cache em memória: nunca faz I/O, subprocesso ou chamada D-Bus síncrona.
2. Eventos são incrementais, tipados, ordenados por revision e enviados somente quando o estado relevante muda.
3. Comandos representam intenção do usuário, nunca callbacks que executam lógica de sistema na UI.
4. Cada integração tem ServiceStatus próprio; ausência de daemon, dispositivo, token, portal ou permissão não derruba o shell.
5. Serviços lazy iniciam por demanda/subscription. Capability check é puro e não instancia singleton nem dispara request.
6. Subscription é dona de um watcher; ao cair a última subscription, parar polling/trabalho caro quando a semântica original permitir. Um watcher compartilhado por serviço.
7. Operações longas publicam progresso/resultado e podem ser canceladas quando seguro. Pedido aceito não significa efeito concluído.
8. Payloads e erros não carregam senha, OAuth token, conteúdo privado de clipboard ou stderr bruto. A regra de não expor título de janela não pode ser global: o original entrega títulos a Overview/Launcher/Dock, enquanto ScreenTime persiste somente a classe. Cada domínio deve declarar minimização, retenção e superfície autorizada; não persistir títulos em ScreenTime.
9. Tipos não contêm GTK, GObject, QML, JavaScript ou tipos de Quickshell.
10. Todos os tipos evoluem aditivamente; campos opcionais/unknown preservam compatibilidade com settings e dados persistidos.

## 2. Fachada pública

~~~rust
pub struct AppHandle { /* cloneable handle; implementation private */ }

pub struct AppSession {
    pub snapshot: AppSnapshot,
    pub events: EventStream,
    _lease: SubscriptionLease,
}

pub struct SubscriptionRequest {
    pub services: BTreeSet<ServiceId>,
}

impl AppHandle {
    // Snapshot cacheado, sem I/O.
    pub fn snapshot(&self) -> AppSnapshot;

    // Snapshot cacheado + stream; cursor/replay e reconciliação ainda pendentes.
    // O conjunto define quais serviços lazy devem iniciar/emitir eventos.
    pub fn connect(&self, request: SubscriptionRequest) -> AppSession;

    // Assíncrono; retorna receipt quando aceito/rejeitado, não quando efeito termina.
    pub async fn dispatch(
        &self,
        command: AppCommand,
    ) -> Result<CommandReceipt, CommandError>;
}
~~~

EventStream implementa Stream<Item = AppEvent>. Seu tipo concreto e integração GLib serão decididos com Codeína; o contrato exige cancelamento automático ao drop da sessão/lease. SubscriptionRequest é referência contada por ServiceId; watchers são compartilhados e a última lease interrompe polling quando permitido. A API ainda precisa especificar como `connect()` coordena serviços assíncronos: snapshot cacheado + cursor/revision do stream, estados `Starting`, emissão de estado inicial e recuperação de eventos perdidos após overflow/desconexão. Não prometer atomicidade de dados ainda não carregados. O frontend pode trocar interesses ao abrir/fechar superfícies; deve consumir o stream no main context sem bloquear GTK. O backend nunca recebe widget ou callback GTK.

## 3. Snapshot e status

~~~rust
pub struct AppSnapshot {
    pub schema_version: u32,
    pub revision: u64,
    pub captured_at: SystemTime,
    pub settings: SettingsSnapshot,
    pub compositor: CompositorSnapshot,
    pub outputs: Vec<OutputSnapshot>,
    pub shell: ShellSnapshot,
    pub services: BTreeMap<ServiceId, ServiceStatus>,
    pub state: DomainSnapshot,
}

pub struct ServiceStatus {
    pub availability: Availability,
    pub lifecycle: Lifecycle,
    pub capabilities: BTreeSet<ServiceCapability>,
    pub since: Option<SystemTime>,
    pub last_success: Option<SystemTime>,
    pub last_error: Option<SafeError>,
}
~~~

Availability: Available, Degraded, Unavailable, Disabled, NotConfigured, PermissionDenied, RateLimited.<br>
Lifecycle: Stopped, Starting, Running, Stopping, Failed.

ServiceId deve cobrir os 61 serviços QML ativos catalogados, inclusive serviços opcionais. ServiceCapability é enum tipado (ex.: AudioVolume, AudioMute, AudioSinkSelect, NetworkScan, NetworkConnect, CameraEffects, FaceEnrollment, ScreenCapture, WallpaperMotion, PackageInstall, CalendarRead, CalendarWrite, ProfileImportExport, ColourPick, Spectrum, GameRecords, PetCare, ControlLayout, ShortcutCatalogue). Não inferir capability somente pela presença de binário: probe deve refletir efeito real possível.

SafeError contém ErrorCode estável, mensagem curta/localizável e retryable; nunca texto cru de processo ou segredo.

## 4. Tipos de domínio mínimos

~~~rust
pub struct CompositorSnapshot {
    pub kind: CompositorKind,
    pub connected: bool,
    pub version: Option<String>,
    pub focused_output: String,
    pub focused_window: Option<WindowSnapshot>,
    pub workspaces: Vec<WorkspaceSnapshot>,
    pub windows: Vec<WindowSnapshot>,
    pub niri: Option<NiriServiceState>,
}

pub struct OutputSnapshot {
    pub id: OutputId,
    pub name: String,
    pub description: Option<String>,
    pub geometry: Rect,
    pub scale: f64,
    pub active: bool,
    pub internal: bool,
    pub virtual_output: bool,
    pub focused: bool,
    pub effective_primary: bool,
    pub hosts_live_island: bool,
}

pub struct ShellSnapshot {
    pub live_island_output: Option<OutputId>,
    pub do_not_disturb: bool,
    pub game_mode: bool,
    pub active_panel: Option<PanelId>,
}

pub struct SettingsSnapshot {
    pub schema_version: u32,
    pub revision: u64,
    pub values: BTreeMap<SettingKey, SettingValue>,
}

pub enum SettingValue {
    Null, Bool(bool), Integer(i64), Number(f64), String(String),
    List(Vec<SettingValue>), Object(BTreeMap<String, SettingValue>),
}
~~~

Multi-monitor: outputs virtuais (incluindo Virtual-1) são outputs normais; primary efetivo atual é focused output com fallback para primeiro output ativo, não um primary independente do Niri. O handover é estado explícito: enquanto painel está expandido, a ilha live não troca de output; ao fechar, assenta no output desejado. O contrato de lifecycle/ordenação dessa transição e a regra quando não há output elegível precisam ser derivados da fonte ativa antes da implementação. Backend expõe qual output é live; frontend desenha barra/superfícies por output.

DomainSnapshot é struct tipada, não JSON arbitrário. Contratos draft documentais para os grupos Compositor/Monitor/Idle/Session/Lock/Polkit, Font/Emoji/Machine/Version/Sunset/Applications e Niri/Settings/Audio/Network/Bluetooth/Battery/Brightness estão nas seções 12.1–12.3. Outros serviços e lifecycle genérico ainda não estão fechados. Cada domínio deve declarar estado vazio, não inicializado, indisponível, degradado e erro sem confundir ausência com zero. `Option` isoladamente não distingue esses estados.

## 5. Eventos

~~~rust
pub struct AppEvent {
    pub revision: u64,
    pub occurred_at: SystemTime,
    pub kind: AppEventKind,
}

pub enum AppEventKind {
    SnapshotReady(AppSnapshot),
    ServiceStatusChanged { service: ServiceId, status: ServiceStatus },
    SettingsChanged { revision: u64, keys: Vec<SettingKey> },
    OutputsChanged { outputs: Vec<OutputSnapshot> },
    CompositorChanged(CompositorSnapshot),
    ShellChanged(ShellSnapshot),
    DomainChanged(DomainEvent),
    CommandProgress { id: CommandId, progress: CommandProgress },
    CommandFinished { id: CommandId, result: Result<CommandResult, SafeError> },
}
~~~

DomainEvent usa enum por domínio (AudioChanged, NetworkChanged, BluetoothChanged, BatteryChanged, BrightnessChanged, NotificationReceived/Dismissed, MediaChanged, PrivacyChanged, ClipboardChanged, CaptureChanged, RecorderChanged, AccountChanged, ProfileChanged, CalendarChanged, ControlsChanged, PickerChanged, CameraChanged, FaceChanged, StatsChanged, CavaChanged, NotesChanged, TasksChanged, TimerChanged, VmChanged, PackagesChanged, UpdatesChanged, UsageChanged, WeatherChanged, MarketsChanged, GithubChanged, ScreenTimeChanged, ServersChanged, ThemeChanged, WallpaperChanged, GamesChanged, PetChanged, BarLayoutChanged, LauncherChanged, ApplicationsChanged, DesktopChanged, DockChanged, DeckChanged, WorkspacesChanged, NiriChanged, CompositorServiceChanged, MonitorChanged, IdleChanged, SessionChanged, LockChanged, PolkitChanged, FontChanged, EmojiChanged, MachineChanged, VersionChanged, SunsetChanged). Settings permanece `AppEventKind::SettingsChanged { revision, keys }`; não duplicar como DomainEvent. A existência e granularidade desses eventos novos é proposta para o contrato Rust, derivada de propriedades/sinais observados, não evento literal do QML. Eventos de janela/workspace/output preservam IDs Niri e ordem original. Evento de valor idêntico não deve ser emitido.

## 6. Comandos tipados

~~~rust
pub enum AppCommand {
    Settings(SettingsCommand),
    Niri(NiriCommand),
    Audio(AudioCommand),
    Network(NetworkCommand),
    Bluetooth(BluetoothCommand),
    Brightness(BrightnessCommand),
    Notifications(NotificationCommand),
    Media(MediaCommand),
    Clipboard(ClipboardCommand),
    Capture(CaptureCommand),
    Recorder(RecorderCommand),
    Account(AccountCommand),
    Profiles(ProfileCommand),
    Calendar(CalendarCommand),
    Controls(ControlsCommand),
    Picker(PickerCommand),
    Face(FaceCommand),
    Camera(CameraCommand),
    Launcher(LauncherCommand),
    Panel(PanelCommand),
    Notes(NotesCommand),
    Tasks(TasksCommand),
    Timer(TimerCommand),
    Packages(PackagesCommand),
    Updates(UpdatesCommand),
    Vm(VmCommand),
    Servers(ServersCommand),
    ScreenTime(ScreenTimeCommand),
    Theme(ThemeCommand),
    Wallpaper(WallpaperCommand),
    Desktop(DesktopCommand),
    Dock(DockCommand),
    Deck(DeckCommand),
    Pet(PetCommand),
    Games(GamesCommand),
    Session(SessionActionCommand),
    Lock(LockCommand),
    Polkit(PolkitCommand),
    Fonts(FontCommand),
    Emoji(EmojiCommand),
    Machine(MachineCommand),
    Version(VersionCommand),
    Sunset(SunsetCommand),
    Applications(ApplicationsCommand),
    LegacyIpc(LegacyIpcCommand),
    Cancel { command_id: CommandId },
}
~~~

Comandos iniciais auditados (nomes são tipos de domínio, não nomes obrigatórios de métodos internos):
- Niri: FocusWorkspace, FocusWindow, CloseWindow, ToggleFloating, FocusOutput e ações documentadas pelo protocolo real.
- Audio: SetVolume, SetMuted, SelectSink, SelectSource, SetSourceMuted.
- Network: SetRadio, Scan, Connect{network, credential: SecretInput}, Disconnect, Forget. SecretInput é zeroized e nunca Debug/Serialize/log.
- Bluetooth: SetPowered, StartDiscovery, StopDiscovery, Pair, Connect, Disconnect, Remove.
- Brightness: SetPercent, Step.
- Session: ação interna observada lock/suspend/logout/reboot/shutdown; confirmar o gate de UI por componente. Não há evidência nesta rodada para hibernate, lid policy ou um target CLI power.
- Notifications: SetDoNotDisturb, Dismiss, InvokeAction, ClearHistory.
- Media: Play, Pause, Toggle, Next, Previous, Seek, SetPosition.
- Clipboard: Search, CopyEntry, DeleteEntry, ClearHistory; wipe-on-lock é policy persistente.
- Capture: Open{shape,destination}, CaptureScreen, CaptureText, Cancel.
- Recorder: Start, Stop, Toggle, SetAudio.
- Account: SetDisplayName, SetAvatarPng, ClearAvatar.
- Face: Refresh, Add{label}, Remove{model_id}, TryUnlock. PAM e pkexec são fluxos separados.
- Camera: SetEnabled, SetEffect, SetReaction, Snap.
- Launcher: Open, Close, SetMode, ActivateResult.
- Profiles: Create, Duplicate, Rename, Remove, Switch, Import{source: FileGrant}, Export{destination: FileGrant}. Import rejeita formato futuro/inválido e reporta chaves ignoradas; export preserva wallpaper/palette/settings profileKeys e exclui machine/user data.
- Calendar: RefreshMonth, OpenCalendar, NewEvent, OpenEvent; serviço opcional usa dcal como dono dos dados e não cria store próprio.
- Controls: SetTopRow, MoveTopItem, ToggleTopItem, AddBlock, RemoveBlock, MoveBlock, ResizeBlock, CycleBlockSize, SetBlockToggles, SetBlockDoors, RestoreDefaults, SetGridSize. Algoritmo nearest-free e collision pertence ao backend.
- Picker: Pick{settle_ms}, Cancel; hyprpicker devolve hex validado, wl-copy recebe sem newline.
- Games: Open, Leave, RecordScore. Pet: Feed, Play, Rename, BringOut. Shortcut catalogue is read-only because the original reports compositorBindingsEditable=false; do not add a keybinding editor or write Niri KDL.
- Notes: Add, Append, Archive, Remove; Tasks: Add, Move, SetDue, Complete, Remove; Timer: Start, Pause, Resume, Cancel.
- Packages: Search, LoadInstalled, Install, Remove, UpgradeAll. Instalação/remoção/upgrade devem abrir terminal interativo como no original, não executar sudo oculto.
- Updates: Refresh. VM: List, Start, Open, Pause, Resume, Stop, Kill, Remove, Snapshot, SetCores, SetMemory, Create, CancelCreate.
- Servers: Refresh, OpenLocalUrl, CopyLocalUrl, Stop{pid,port,identity}. Stop revalida identidade do processo antes de sinalizar.
- ScreenTime: SubscribeReport, ReadReport{view,date}; Theme: SetPalette, RefreshAdaptive, PushTerminalSettings, SetGreeting; Wallpaper: Scan, Apply, Restore.
- Font: Load; Emoji: Load, StepGroup, StepTone, Remember, Copy; Machine: Refresh; Version: Check, Update, ListFiles, ChangeFiles{Restore|Keep|Reset,paths}; Sunset: Toggle, SetTemperature{kelvin}; Applications: Refresh, LaunchDesktop{desktop_id}. Estes nomes são comandos tipados propostos para operações observadas; receipts permanecem limitados ao que o adapter efetivamente observa.
- LegacyIpc: enum de targets/verbs de IPC_AUDIT.md; argumentos estruturados e validados, nunca shell string. Manter o formato de saída CLI e erros observáveis.

## 7. Resultado, erros e segurança

~~~rust
pub struct CommandReceipt {
    pub id: CommandId,
    pub accepted_at: SystemTime,
    pub state: CommandState, // Queued | Running | Completed
}

pub struct CommandError {
    pub code: ErrorCode,
    pub service: Option<ServiceId>,
    pub retryable: bool,
    pub safe_message: String,
}
~~~

dispatch retorna erro imediato para comando inválido, capability ausente, permissão negada ou serviço não inicializável. Operação que foi aceita publica CommandProgress e CommandFinished; sucesso só após efeito confirmado. Um subprocesso iniciado não equivale a sucesso. Erros detalhados ficam em log estruturado redigido; UI recebe mensagem segura. Cancelamento é best-effort e cada comando declara se pode ser cancelado.

## 8. Semântica de subscriptions e lazy loading

- connect(request) adquire leases e retorna snapshot cacheado + stream. Não há garantia especificada de fronteira atômica, replay, ausência de perda ou reconciliação; esses detalhes continuam bloqueados por API-04.
- EventStream é multicast por serviço, não watcher por widget; a sessão libera leases automaticamente ao drop.
- Subscribe inicia o serviço se necessário; drop libera referência. Uma subscription não pode impedir outra de receber eventos.
- Painéis ocultos liberam subscriptions visuais; serviços com atividade de background obrigatória (lock/session/clipboard policy/recorder quando ativo) seguem lifecycle de domínio, não do painel.
- Camera server só inicia quando cameraEnabled e capabilities prontas; webcam abre somente quando reader da virtual camera aparece.
- ScreenTime recorder segue lifecycle de sessão, mas relatórios só são calculados enquanto painel/chip/desktop tem follower.
- Stats/Weather/Markets/Claude/Codex/Github/Updates respeitam watcher count e intervalo do original.
- Cava só roda com subscriber ou linger de 2 s; game mode interrompe. Subscribe/release de Cava é parte do ciclo de vida do EventStream, não comando de UI. Pet company XP depende de placement no bar, lock state e mood, não só de watcher count.
- Games e Pet têm persistência própria (games.json/pet.json); ProfileService deliberadamente não exporta esses dados.
- ModuleService.has() vira capability query puro; ensure_started() explícito evita side effects ocultos.

## 9. Compatibilidade e pontos que exigem revisão

- IPC/CLI legado: targets e verbos em IPC_AUDIT.md são parte da compatibilidade. Alguns retornos significam pedido aceito, não efeito concluído; o novo contrato distingue receipt e conclusão sem mudar a saída externa.
- Settings: preservar as 108 chaves, valores desconhecidos, perfis e machine keys; migração não escreve em quickshell e requer backup/consentimento.
- Niri: implementar conforme protocolo real e manter outputs virtuais; não inventar mensagens.
- Shell reload, layer-shell no Niri, tray backend, capture/portal e runtime async ainda exigem decisões separadas.
- Contrato ainda não cobre os detalhes de todos os 61 serviços e superfícies UI. Extensões são obrigatórias antes de implementar cada domínio restante; não considerar esta draft uma aprovação de Codeína.

## 10. Review com Codeína

Antes de congelar nomes, payloads ou transporte compartilhado:
1. Codeína revisa tipos e mapeamento com design.md e consumidores QML.
2. Frontend confirma estados visuais que precisam existir no snapshot/event stream.
3. Backend incorpora feedback sem transferir lógica de sistema para widgets.
4. Atualizar ARCHITECTURE.md, este contrato e fixtures na mesma mudança.
5. Registrar decisão de runtime/transport em DECISIONS.md. Até lá, este documento é draft e não autoriza escolha unilateral de Relm4/Tokio.

## 11. Fixtures de frontend

Fixtures sintéticas em docs/fixtures:
- app_snapshot.mock.json: snapshot inicial com dois outputs (eDP-1 e Virtual-1), estados disponíveis/indisponíveis e dados de exemplo.
- app_events.mock.jsonl: eventos incrementais de domínio, disponibilidade e ciclo de comando.

Serialização de fixtures usa nomes snake_case, enum discriminant type e payload data quando aplicável; timestamps RFC3339 com offset. Os exemplos são deliberadamente sintéticos e cobrem apenas uma fatia do DomainSnapshot; não representam a máquina, não consultam serviços e não contam como funcionalidade. Codeína pode usá-los para compor a UI enquanto os providers reais são implementados. Qualquer mudança de shape exige atualizar este contrato e ambos os fixtures na mesma alteração.

## 12. Primitivos e forma dos resultados

~~~rust
pub struct OutputId(pub String);
pub struct WorkspaceId(pub u64);
pub struct WindowId(pub String);
pub struct NotificationId(pub u32);
pub struct SettingKey(pub String);
pub struct CommandId(pub u64);

// O frontend cria estes grants após escolha explícita no file chooser/portal.
// O backend não recebe path arbitrário de arquivo do usuário.
pub struct FileGrant { /* opaque, revocable file capability */ }

pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub enum Availability {
    Available, Degraded, Unavailable, Disabled,
    NotConfigured, PermissionDenied, RateLimited,
}

pub enum Lifecycle { Stopped, Starting, Running, Stopping, Failed }

pub struct SafeError {
    pub code: ErrorCode,
    pub safe_message: String,
    pub retryable: bool,
}

pub enum CommandStage {
    Validating, WaitingForPermission, Starting, Running, Finalizing,
}

pub struct CommandProgress {
    pub stage: CommandStage,
    pub fraction: Option<f32>,
    pub safe_message: Option<String>,
}

pub enum CommandOutcome {
    Succeeded { safe_message: Option<String> },
    Cancelled,
    PartiallySucceeded { safe_message: String, warnings: Vec<String> },
}
~~~

Identifiers são estáveis durante a vida da entidade; IDs Niri/DBus podem ser encapsulados em string quando protocolo não oferece inteiro estável. Rect usa coordenadas lógicas do compositor, não pixels físicos. Fração de progresso é None quando não mensurável e, quando presente, fica em 0.0..=1.0. CommandOutcome só é emitido depois de confirmar o efeito ou o cancelamento.

### Enum de serviço e erros estáveis

~~~rust
pub enum ServiceId {
    Compositor, Monitor, Niri, Idle, Session, Lock, Polkit, Shortcut,
    Settings, Theme, Profile, Font, Wallpaper, Sunset, Picker, Account,
    Module, Controls, Osd, Notification, Tray, Launcher, Applications,
    Audio, Battery, Bluetooth, Brightness, Network, System, Privacy,
    Camera, Calendar, Clipboard, Emoji, Media, Lyrics, Notes, Tasks,
    Timer, ScreenTime, Desktop, Dock, Deck, Capture, Recorder, Stats,
    Cava, Claude, Codex, Github, Markets, Weather, Updates, Packages,
    Version, Machine, Servers, Vm, Games, Pet, Face,
}

pub enum ErrorCode {
    InvalidArgument, Unsupported, Unavailable, MissingDependency,
    PermissionDenied, AuthenticationCancelled, Timeout, Disconnected,
    RateLimited, NotFound, Conflict, Io, Protocol, ExternalFailure,
    Cancelled, Internal,
}
~~~

Os códigos são estáveis e não incluem mensagens específicas de ferramentas. Mensagens seguras podem ser localizadas na UI; logs técnicos usam o mesmo code + service + correlation id com segredos redigidos.

### Tipos de comando e segredo

~~~rust
pub enum CommandState { Queued, Running, Completed }

pub struct CommandResult {
    pub outcome: CommandOutcome,
}

pub struct SecretInput {
    // Opaque zeroizing buffer; never Clone/Debug/Serialize/Display.
    // Consumed by the command and erased immediately after provider use.
}
~~~

SecretInput é construído somente pelo fluxo de credenciais da UI; nunca entra em SettingsSnapshot, fixtures, logs, IPC persistido ou histórico.

### Shape obrigatório de DomainSnapshot

~~~rust
pub struct DomainSnapshot {
    pub fonts: Option<FontState>,
    pub emoji: Option<EmojiState>,
    pub machine: Option<MachineState>,
    pub version: Option<VersionState>,
    pub sunset: Option<SunsetState>,
    pub applications: Option<ApplicationsState>,
    pub compositor_service: Option<CompositorServiceState>,
    pub monitor: Option<MonitorState>,
    pub idle: Option<IdleState>,
    pub session: Option<SessionState>,
    pub lock: Option<LockState>,
    pub polkit: Option<PolkitState>,
    pub audio: Option<AudioState>,
    pub network: Option<NetworkState>,
    pub bluetooth: Option<BluetoothState>,
    pub battery: Option<BatteryState>,
    pub brightness: Option<BrightnessState>,
    pub notifications: Option<NotificationState>,
    pub media: Option<MediaState>,
    pub tray: Option<TrayState>,
    pub privacy: Option<PrivacyState>,
    pub clipboard: Option<ClipboardState>,
    pub capture: Option<CaptureState>,
    pub recorder: Option<RecorderState>,
    pub account: Option<AccountState>,
    pub profiles: Option<ProfileState>,
    pub calendar: Option<CalendarState>,
    pub controls: Option<ControlsState>,
    pub picker: Option<PickerState>,
    pub camera: Option<CameraState>,
    pub face: Option<FaceState>,
    pub stats: Option<StatsState>,
    pub system: Option<SystemState>,
    pub notes: Option<NotesState>,
    pub tasks: Option<TasksState>,
    pub timer: Option<TimerState>,
    pub vm: Option<VmState>,
    pub packages: Option<PackageState>,
    pub updates: Option<UpdateState>,
    pub usage: Option<UsageState>,
    pub weather: Option<WeatherState>,
    pub markets: Option<MarketsState>,
    pub github: Option<GithubState>,
    pub screen_time: Option<ScreenTimeState>,
    pub servers: Option<ServersState>,
    pub theme: Option<ThemeState>,
    pub wallpaper: Option<WallpaperState>,
    pub cava: Option<CavaState>,
    pub games: Option<GamesState>,
    pub pet: Option<PetState>,
    pub shortcuts: ShortcutCatalogueState,
    pub bar: Option<BarState>,
    pub launcher: Option<LauncherState>,
    pub desktop: Option<DesktopState>,
    pub dock: Option<DockState>,
    pub deck: Option<DeckState>,
}
~~~

Option neste rascunho não distingue serviço ainda não iniciado, consulta pendente, dado ausente, integração indisponível e erro. Não usar esse shape como contrato congelado. A diferença deve ser representada no schema concreto por serviço; lista vazia só é válida quando a fonte confirmou lista vazia.

~~~rust
pub struct SubscriptionLease {
    // Private RAII token. Drop decrements per-service subscriber counts.
}
~~~

SubscriptionRequest pode incluir serviços sem UI visível quando um domínio é necessário para uma superfície persistente; core watchers de lock/session são mantidos pelo runtime e não dependem de painel aberto.

### 12.1 Contratos auditados — compositor e ciclo da sessão (rodada 1)

Os schemas abaixo são contratos observados/propostos a partir do QML e scripts citados. Os nomes Rust/eventos são proposta de representação, não eventos literais do QML. A matriz rastreável está em `docs/APP_API_MATRIX.md`; classificação item a item e plano de validação isolada estão em `docs/APP_API_REVIEW.md` (rodada 3). Status documental: Compositor, Monitor e Session têm contrato completo da superfície que o original torna observável; Idle, Lock e Polkit permanecem incompletos. Nenhum dos seis está validado em runtime.

~~~rust
pub struct CompositorServiceState { pub version: String, pub config_path: String, pub loaded: bool, pub last_error: String }
pub struct MonitorState { pub outputs: Vec<MonitorOutput>, pub loaded: bool, pub refresh_pending: bool, pub last_error: String,
    pub primary_name: String, pub internal_name: String }
pub struct MonitorOutput { pub name: String, pub description: String, pub make: String, pub model: String,
    pub serial: String, pub x: i64, pub y: i64, pub width: u32, pub height: u32, pub refresh_rate: f64,
    pub mode: String, pub scale: f64, pub transform: String, pub disabled: bool, pub vrr: bool,
    pub vrr_supported: bool, pub resolutions: Vec<MonitorResolution> }
pub struct MonitorResolution { pub width: u32, pub height: u32, pub refreshes: Vec<f64> }
pub struct IdleState { pub lock_after_minutes: u32, pub screen_after_minutes: u32, pub suspend_after_minutes: u32,
    pub lock_idle: bool, pub screen_idle: bool, pub suspend_idle: bool }
pub struct SessionState { pub sleep_pending: bool, pub suspend_when_locked: bool }
pub struct LockState { pub locked: bool, pub secure: bool, pub authenticating: bool, pub message: String,
    pub failed: bool, pub leaving: bool, pub awake: bool, pub awake_for_ms: u32, pub shot_ready: bool,
    pub shot_serial: u64, pub face_ready: bool, pub face_scanning: bool, pub face_matched: bool,
    pub face_misses: u32, pub face_quiet_until_unix_ms: u64 }
pub struct PolkitState { pub asking: bool, pub active: bool, pub registered: bool, pub failed: bool, pub tried: u8, pub prompt: Option<PolkitPromptSnapshot> }
pub struct PolkitPromptSnapshot {
    pub icon_name: String,
    pub message: Option<String>, pub action_id: Option<String>,
    pub response_visible: bool, pub response_required: bool,
    pub input_prompt: Option<String>, pub supplementary_is_error: bool,
    pub supplementary_message: Option<String>, pub is_completed: bool,
    pub is_successful: bool, pub is_cancelled: bool, pub failed: bool,
}
pub enum IdleWatch { Lock, Screen, Suspend }
pub struct IdleTransition { pub watch: IdleWatch, pub idle: bool }
pub struct SessionChanged { pub sleep_pending: bool, pub suspend_when_locked: bool }
pub struct MonitorChanged { pub state: MonitorState }
pub struct LockPublicState {
    pub locked: bool, pub secure: bool, pub authenticating: bool, pub message: String,
    pub failed: bool, pub leaving: bool, pub awake: bool, pub shot_ready: bool,
    pub shot_serial: u64, pub face_ready: bool, pub face_scanning: bool,
    pub face_matched: bool, pub face_misses: u32,
}
pub enum SessionActionId { Lock, Suspend, Logout, Reboot, Shutdown }
pub enum SessionActionCommand { Run { action: SessionActionId } }
pub enum LockCommand { RequestLock }
pub enum PolkitCommand { Submit { secret: SecretInput }, Cancel }
~~~

Os campos de `MonitorOutput` reproduzem os valores produzidos pelo script, incluindo defaults que colapsam ausência/null em zero/strings. `typed` é segredo efêmero e nunca pertence a snapshots, eventos, logs ou fixtures. O payload Polkit usa `Option<PolkitPromptSnapshot>` em lugar do objeto runtime; `asking` e `prompt.is_some()` mantêm semânticas distintas (asking também requer agent ativo). Os metadados QML locais listam mais campos no flow (`identities`, `selectedIdentity`); a UI atual não os consome e a forma de `Identity` não está descrita no `.qmltypes`, portanto sua modelagem fica pendente, sem expor os objetos. O bool `IdleState.*_idle` não tem valor inicial documentado pelo QML e permanece pendente. `LockPublicState` é projeção segura para consumidores; timers, shellQuiet, facePam e pendingCapture pertencem ao estado interno do serviço e não são todos consumidos externamente.

#### CompositorService

- **Estado inicial:** `version=""`, `loaded=false`, `lastError=""`. `configPath` é calculado de `XDG_CONFIG_HOME`, ou `HOME + "/.config"`, e `/niri/config.kdl`.
- **Start/refresh:** `Component.onCompleted` chama `load()`, que inicia `niri --version`. Não há polling, watcher ou refresh automático. A fonte não define coalescing/rejeição quando chamado com processo ativo.
- **Atualização:** stdout concluído trimado substitui `version`; `loaded` vira `version != ""`. stderr não vazio substitui `lastError`; stderr vazio não limpa erro. Sucesso não limpa `lastError`. Não há handler de exit code/status: stdout vazio/falha sem stderr não define transição adicional.
- **Erro/recuperação:** versão anterior pode permanecer após falha posterior, pois `load()` não limpa campos. Exit != 0, timeout, binário ausente e stderr vazio não viram erro tipado. `Process` metadata declara `exited`/`errorOccurred`, mas o adapter não conecta handlers para esses sinais. Não inventar retry nem classificar `loaded=false` como indisponível versus erro.
- **Stop:** serviço não declara cancelamento nem teardown do processo.
- **Consumidores:** `settings/WindowsSection.qml` apresenta versão com fallback visual “Niri”; `settings/InputSection.qml` usa `configPath`; NiriMod é fluxo separado que edita KDL.
- **Comando/evento:** nenhuma mutação/comando de domínio. `CompositorServiceChanged { state }` da enum global pode transportar alterações observadas, mas não é evento original. Não prometer `LoadFailed` inexistente.
- **Evidência:** `services/CompositorService.qml`, `settings/WindowsSection.qml`, `settings/InputSection.qml`.
- **Pendente/runtime:** a fonte não determina resultados de processo fora de stdout/stderr. É necessário executar binário ausente, exit não-zero com/sem stderr e falha após sucesso para fixar compatibilidade; até lá o adapter 1:1 deve preservar as regras acima, sem afirmar estado adicional.

#### MonitorService

- **Estado inicial:** `monitors=[]`, `loaded=false`, `refreshPending=false`, `lastError=""`. O código não distingue inicialmente outputs vazios de ainda não carregados sem `loaded`.
- **Start/refresh:** inicia em `Component.onCompleted` e quando `MonitorsSection` abre. `WorkspacesChanged`, `Quickshell.screensChanged` e `screen.geometryChanged` reiniciam debounce de 120ms. A fonte informa que Niri 26.04 não publica evento próprio de mudança de output.
- **Adapter e payload:** `scripts/monitors.py get` invoca `niri msg --json outputs`, timeout 5s. Aceita payload dict direto, `Err`, `Ok`, `Outputs`; resultado final precisa ser dict. Ignora entradas cujo valor output não seja dict. JSON/campos numericamente inválidos abortam leitura. Para cada saída inclui `name`, `description` (make/model/serial unidos, fallback name), make/model/serial, x/y, logical width/height, refreshRate (mHz/1000 arredondado 3 casas), `mode` (resolução física e `@rate` com 3 casas, ou `off`), scale, transform, disabled (logical nulo), vrr, vrrSupported, resolutions agrupadas por resolução com rates. Defaults explícitos: x/y/width/height e dimensões físicas 0; scale 1; transform `Normal`; strings vazias; bool false; arrays vazios. A ordem é a ordem de iteração do dict Python após parse JSON.
- **Sucesso:** stdout vazio/whitespace é ignorado. Parse válido substitui toda a lista, `loaded=true`, limpa lastError. Falha mantém lista e loaded anteriores. `refreshPending` indica um refresh adicional coalescido. Durante reader ativo, qualquer quantidade de load vira um único refresh posterior via `Qt.callLater`, tanto em exit 0 como não-zero.
- **Erros:** stderr não vazio substitui lastError; JSON inválido registra String(error); exit !=0 sem erro usa `niri msg outputs exited (<code>)`. Script retorna 1 para RuntimeError/ValueError, 2 para uso inválido. Exit 0 com stdout vazio não gera erro. `Process.errorOccurred` existe na metadata instalada, mas não é conectado pelo adapter. Não há retry periódico, timestamp, classificação, ou indicador stale; recuperação é apenas nova chamada por gatilhos acima.
- **Derivações:** primaryName é focusedOutput se presente; senão primeiro não-disabled; senão `""`. `primaryScreen` procura nome nas `Quickshell.screens`, depois usa `screens[0]`, senão null. `internalName` é primeira saída cujo nome começa eDP/LVDS/DSI. `monitorFor` aceita name ou description; `descriptionFor` retorna vazio se ausente; `isPrimary` compara screen.name ao primaryName.
- **Stop/comandos:** não há target IPC nem teardown/cancel explícito; `load()` é interno/read-only. Perfis persistentes pertencem a Kanshi/KDL; serviço não grava configuração Niri.
- **Consumidores:** Settings MonitorsSection; shell e multi-monitor handover; DesktopService/Desktop; CaptureService usa primary/output, sem resolver Capture Window.
- **Eventos tipados propostos:** variante global `MonitorChanged` com snapshot tipado (outputs, derivados, loaded, refreshPending e lastError); não separar `MonitorRefreshFailed`, porque lastError é apenas campo textual e a fonte não tem evento de erro. Carga/refresh status não tem sinal QML dedicado.
- **Evidência:** `services/MonitorService.qml`, `scripts/monitors.py`, `settings/MonitorsSection.qml`, `shell.qml`, `services/NiriService.qml`, `services/DesktopService.qml`, `services/CaptureService.qml`.
- **Pendente/runtime:** ordem de corrida entre Niri snapshot e hotplug; referência a screen removida; variações de schema por versão Niri; stdout vazio; processo/helper indisponível; distinguishing lista legitimamente vazia de refresh não concluído. O last-good em falha é comportamento confirmado em código.

#### IdleService

- **Lifecycle/estado:** singleton com três `IdleMonitor` (lock/screen/suspend), criado pelo uso no shell. Valores live vêm de `SettingsService.idleLock`, `idleScreen`, `idleSuspend`, minutos convertidos em segundos; cada monitor `enabled` se timeout >0, timeout zero significa nunca e todos `respectInhibitors=true`. Idle não é persistido. A fonte não define estado inicial explícito de `isIdle`.
- **Transições:** lock watch chama LockService.lock somente em mudança para idle=true; screen watch chama `screen(false)` quando idle e `screen(true)` quando volta ativo; isto invoca NiriService `power-off-monitors`/`power-on-monitors`. Suspend watch chama SessionService.run("suspend") apenas ao virar idle. Atualizar setting altera binding live; efeito sobre contagem de timeout ativa fica delegado ao IdleMonitor/Wayland.
- **Erros/recuperação:** sem onError/status de protocolo/inhibitor. Falha DPMS é responsabilidade não reportada pelo NiriService aqui. Reativar monitor não cancela uma sessão suspend já solicitada; ações são fire-and-forget.
- **Stop:** sem método; vida segue singleton/runtime. Sem eventos próprios de UI.
- **Consumidores:** SettingsService como fonte; LockService, NiriService, SessionService como efeitos; ScreenTimeService lê screenWatch.enabled/isIdle; shell mantém serviço alcançável.
- **Contrato:** command público próprio não observado. Variante global `IdleChanged` com payload discriminado por watch (`Lock|Screen|Suspend`, `idle`) e configuração, preservando as três transições independentes; efeitos apontam para contratos dos outros serviços.
- **Evidência:** `services/IdleService.qml`, `shell.qml`, `services/SettingsService.qml`, `services/ScreenTimeService.qml`, `services/NiriService.qml`.
- **Pendente/runtime:** suporte e erro de ext-idle-notify no compositor; reconfiguração live enquanto já idle; inhibitors e corrida em suspend; resultado de DPMS. Até runtime, não inventar falhas recuperadas nem valor inicial do bool idle.

#### SessionService

- **Catálogo UI (ordem original):** `lock` (icon `󰌾`, label `Lock`, destructive false), `suspend` (`󰤄`, `Suspend`, false), `logout` (`󰗽`, `Log out`, true), `reboot` (`󰜉`, `Restart`, true), `shutdown` (`󰐥`, `Shut down`, true). Campos são dados literais de apresentação; não localizar/normalizar sem decisão. Confirmações ficam nos componentes, não no serviço.
- **Estado:** `sleepPending=false`, `suspendWhenLocked=false`. `sleepHold` abre systemd-inhibit delay e deixa `cat` aguardando stdin; `sleepWatch` inicia dbus-monitor system bus e parseia apenas linha `boolean true|false` para PrepareForSleep.
- **Comandos e efeitos:** `run(lock)` chama LockService.lock. `run(suspend)` executa systemctl suspend imediatamente se secure; senão marca `suspendWhenLocked`, reinicia timer 5s e pede lock; evento secure true consome a flag/timer e chama systemctl. Timeout limpa flag e avisa, sem suspender. `run(logout)` chama NiriService.runAction([quit,--skip-confirmation]); reboot/shutdown iniciam systemctl reboot/poweroff. ID desconhecido somente warning. Chamador não recebe result.
- **PrepareForSleep:** true com secure já verdadeiro apenas solta inhibitor; true sem secure seta sleepPending e pede lock; secure true limpa sleepPending e solta inhibitor. `false` limpa sleepPending e reinicia sleepHold. Isso é separado de ação suspend manual.
- **Falhas/concorrência:** runner único recebe command e running=true sem verificar processo ativo; sem stdout/stderr/exit/error handlers, embora `Process` disponibilize sinais correspondentes na metadata instalada. Não há supervisão de spawn/fim de `systemd-inhibit` ou `dbus-monitor`, nem recuperação de watcher/inhibitor. Chamadas repetidas/concorrentes podem reutilizar o mesmo runner; semântica de Quickshell nesse caso não está especificada no QML. Não existe confirmação de sucesso de shutdown/logout/suspend.
- **Stop:** nenhum shutdown explícito. O código comenta que stdin fecha junto do shell; ordem de encerramento e liberação real do inhibitor dependem runtime.
- **Consumidores:** SessionPanel, ButtonsBlock, TopButtons, LockPower, IdleService e LockService. Settings SessionSection não chama esse serviço; configura outros domínios.
- **Contrato:** comando interno `Run { action: SessionActionId }`; sem CLI alvo verificado. Receipt se usado só pode significar solicitação aceita localmente, não efeito concluído. Variante global `SessionChanged` transporta `sleepPending` e `suspendWhenLocked`, sem resultado do sistema.
- **Evidência:** `services/SessionService.qml`, `services/IdleService.qml`, `services/LockService.qml`, `services/NiriService.qml`, `bar/island/SessionPanel.qml`, `bar/island/controls/ButtonsBlock.qml`, `bar/island/controls/TopButtons.qml`, `lock/LockPower.qml`, `settings/SessionSection.qml`.
- **Pendente/runtime:** exit/falha dos comandos, runner ocupado, falha/restart dbus-monitor e inhibitor, timing PrepareForSleep, cinco segundos versus limite logind configurado. Não inventar resultado de execução.

#### LockService

- **Significado/estado:** `locked` solicita lock ao compositor; `secure` é a confirmação copiada de `WlSessionLock.secure` por `LockScreen.onSecureStateChanged`. São estados distintos. `typed` é um campo compartilhado entre outputs e contém segredo; não é parte do contrato serializável. `awakeFor=30000ms`; shellQuiet, leaving, authenticating, failed/message, shot/face fields são transitórios.
- **Request/init:** `lock()` é no-op se locked. Limpa mensagem/falha, leaving e estado de descanso; começa verificação local de PAM facial, chama FaceService.refresh, zera shotReady, marca pendingCapture. Se shell não está quiet, espera até 900ms; caso contrário aguarda 32ms. Depois executa captura por output aceito pela regex `[A-Za-z0-9_.-]+`, `timeout 2 grim -t jpeg -q 90`, remove lock-*.jpg anteriores e espera todos os subprocessos. OnExited incrementa shotSerial, define shotReady apenas com exit code zero, eleva locked mesmo em erro, zera contadores de face e inicia PAM login. Falha de screenshot mantém superfície preta. Se subprocesso nunca conclui, o QML não eleva locked.
- **Password/PAM:** `submit` ignora enquanto authenticating/leaving. Campo vazio tenta face scan. Para senha, limpa falha/message; se responseRequired=false, aborta/reinicia PamContext config login, define `Not ready — press Enter again`, não responde; caso contrário authenticating=true e passa string a pam.respond. Completed Success chama release. Recusa, se não leaving/faceMatched, marca failed e mostra `Too many attempts` em MaxTries ou `Wrong password`, depois reinicia conversation. `onError` marca falha/message `Authentication is unavailable`, loga erro e não reinicia automaticamente.
- **Face:** config `velatura-face`, path check literal `/etc/pam.d/velatura-face` e `/usr/lib/security/pam_howdy.so`. faceReady também exige lista vazia ou faces.length>0. Scan exige locked+secure+ready e não leaving/matched/active; wake tenta só acordado. PamMessage indica scanning. Completed é ignorado fora de locked/awake; Success marca matched e libera após 650ms. Recusa após mensagem incrementa misses e emite faceMissed, quiet 1500ms; recusa sem mensagem quiet 5000ms sem incrementar. Após 3 misses não há auto scan até Enter vazio. Face onError só loga; não limpa faceScanning nem informa UI — estado preso é possível e precisa validação.
- **Awake/rest/release:** rouse só locked e !leaving, awake=true, reinicia drowse e tenta wake. Drowse 30s reinicia enquanto authenticating; senão rest. `rest()` para timer, limpa awake/typed/failed/message e aborta face ativa. `release()` idempotente enquanto leaving; após `Theme.durationMorph`, aborta ambos PAMs, para hold, locked=false, chama rest/forget (typed e shotReady false, apaga screenshots) e emite `unlocked`.
- **Consumers:** LockScreen associa `locked` ao WlSessionLock e escreve secure; LockSurface consome output, shotSource, awake/leaving e input wake; LockAccount lê/escreve typed e lê authenticating/failed/message/faceReady; LockIsland usa faceScanning/Matched/Missed/leaving; LockPower usa catálogo Session; shell controla shellQuiet/clipboard; SessionService observa secure; ScreenTime/Pet/Bar leem locked.
- **Comandos/eventos:** `RequestLock` é único comando exterior confirmado; `Commands.qml` retorna `{"ok":true}` logo após chamar lock, não aguarda secure nem PAM. submit/rouse/rest são API interna de UI. Variante global `LockChanged` transporta projeção de estado que exclui `typed`; o sinal literal `unlocked` e `faceMissed` devem ser preservados como subeventos discriminados, não achatados em boolean.
- **Stop/falhas:** não há shutdown explícito nem política de captura/cancelamento no teardown. Remoção de screenshots é chamada em processo independente. `capture`/`faceCheck` tratam `onExited`, mas não `Process.errorOccurred`; confirmar se spawn failure ainda emite saída que permite elevar `locked`. PAM setup, múltiplos prompts, processo grim que não conclui, permissões, output removido e acquire/release ext-session-lock são pendências de runtime.
- **Evidência:** `services/LockService.qml`, `lock/LockScreen.qml`, `lock/LockSurface.qml`, `lock/LockAccount.qml`, `lock/LockIsland.qml`, `lock/LockPower.qml`, `shell.qml`, `Commands.qml`, `services/SessionService.qml`, `services/ScreenTimeService.qml`, `services/PetService.qml`, `bar/Bar.qml`.

#### PolkitService

- **Lifecycle:** LazyLoader `active=true` cria PolkitAgent. `registered` espelha `agent.isRegistered`; `active` espelha `agent.isActive`; `asking = active && flow != null`. AuthenticationRequestStarted limpa `PolkitService.failed` e emite `requested`; `active=false` emite `settled`. A interface QML local (`/usr/lib/qt6/qml/Quickshell/Services/Polkit/quickshell-service-polkit.qmltypes`) declara no flow também `isCompleted`, `isSuccessful`, `isCancelled`, `failed` e sinais `authenticationSucceeded`, `authenticationFailed`, `authenticationRequestCancelled`. O serviço original conecta somente `authenticationFailed`; AuthPanel não lê os demais resultados. Portanto há resultado tipado observável no flow atual, mas `settled` do serviço não o carrega nem deve ser tratado como equivalente.
- **Prompt tipado necessário:** AuthPanel consome `message`, `actionId`, `responseVisible`, `isResponseRequired`, `inputPrompt`, `supplementaryIsError`, `supplementaryMessage`. QML type metadata declara ainda `iconName`, `cookie`, `identities`, `selectedIdentity` e estados terminais acima. `cookie` pode servir somente como token opaco interno enquanto flow vive; não é parte de snapshot/evento público, log ou persistência até lifetime estar confirmado. A UI atual não consome identities/selectedIdentity; `.qmltypes` não define shape de Identity. Nenhum objeto Qt/Quickshell, ponteiro ou handle runtime atravessa a API.
- **Validade/commands:** `submit(secret)` só encaminha ao flow atual quando `isResponseRequired`; UI ignora entrada vazia ou waiting e limpa campo após envio. `cancel()` só chama cancelAuthenticationRequest quando flow existe e !isCompleted. AuthPanel cancela tanto pelo botão quanto onDestruction. Isso não prova cancel concluído nem sucesso/rejeição terminal. Secret jamais vai para persistência/log/fixture.
- **Eventos:** sinais do serviço original são `requested()` e `settled()`; AuthFlow também declara sucesso/falha/cancelamento e mudanças de properties na metadata instalada. O serviço conecta apenas falha, enquanto `flow` deixa properties acessíveis ao QML. `settled` não significa por si só success/cancel/failure; ler os campos terminais do flow enquanto válido preserva a distinção disponível.
- **Contrato de evento global:** `PolkitChanged` é envelope proposto com flags de agente (`registered`, `active`, `asking`, `failed`, `tried`) e prompt tipado opcional. Eventos `requested`, `settled`, `authenticationSucceeded`, `authenticationFailed`, `authenticationRequestCancelled` e `flowChanged` só devem existir na representação Rust se a API original correspondente estiver confirmada na versão alvo; não converter signal de flow em outcome de `settled`.
- **Recuperação de registro:** timer 6000ms roda quando !registered e tried<3. Cada tick soma tried; nos dois primeiros alterna loader false/true. Ao terceiro, executa detached shell que testa `/usr/lib/polkit-kde-authentication-agent-1` executável e então exec; não observa resultado, não confirma inicialização nem reinicia o timer. O comentário QML diz registro assíncrono e reload herda registro; não provaram resultado real.
- **Stop/consumers:** sem shutdown/unregister explícito. AuthPanel lê dados/failed/waiting e submete/cancela; shell/DynamicIsland abrem/fecham por requested/settled; AccountService fornece avatar/initials.
- **Falhas/pendências:** metadata instalada prova campos/sinais, mas não sua semântica temporal. Testar request concorrente/replacement, validade de cookie/flow, cancelamento e sequência de prompts; fluxo de identidades estático continua sem shape tipado. Falha no registro/fallback não é observada pelo serviço. Não inferir `settled` como outcome nem expor objeto runtime.
- **Evidência:** `services/PolkitService.qml`, `bar/island/AuthPanel.qml`, `shell.qml`, `bar/widgets/DynamicIsland.qml`.

#### Comandos conferidos nesta rodada

- lock.lock(): sem argumentos; chama LockService.lock e responde {ok:true} imediatamente, sem esperar secure/autenticação.
- SessionService.run(actionId): API interna, sem target Commands equivalente; não inferir nova CLI.
- Polkit submit(secret)/cancel e Monitor load são internos, sem target CLI.
- CompositorService não tem comando mutável identificado; NiriMod é integração de edição KDL que requer auditoria própria.

### 12.2 Contratos tipados — Font, Emoji, Machine, Version, Sunset e Applications

Contratos derivados dos adapters, scripts e consumidores originais. Os eventos de domínio Rust são envelopes propostos, não sinais literais do QML. Evidências rastreáveis adicionais estão em `docs/APP_API_MATRIX.md`; auditoria de fontes/consumidores em `BACKEND_AUDIT.md`, `INTEGRATION_AUDIT.md`, `UI_SURFACE_AUDIT.md` e `UI_COMPONENT_CATALOG.md`.

~~~rust
pub struct FontState { pub sans: Vec<String>, pub mono: Vec<String>, pub loaded: bool }
pub struct EmojiState { pub items: Vec<EmojiEntry>, pub language: String, pub fresh: bool, pub group: String, pub tone: u8, pub recent: Vec<String> }
pub struct EmojiEntry { pub glyph: String, pub name: String, pub group: String, pub tones: Option<Vec<String>>, pub keywords: String }
pub struct MachineState { pub ready: bool, pub os: String, pub kernel: String, pub host: String, pub wm: String, pub shell: String, pub packages: Option<u64> }
pub struct VersionState { pub version: String, pub branch: String, pub repo: String, pub tree: String, pub answered: bool, pub available: bool, pub checking: bool, pub offline: bool, pub behind: u64, pub ahead: u64, pub upstream: String, pub checkout_branch: String, pub target: String, pub commits: Vec<VersionCommit>, pub checked_at_unix_ms: u64, pub checked_for: String, pub deleted: Vec<String>, pub edited: Vec<VersionEditedFile>, pub changing: bool }
pub struct VersionCommit { pub hash: String, pub subject: String }
pub struct VersionEditedFile { pub path: String, pub waiting: bool }
pub struct SunsetState { pub available: bool, pub restarting: bool, pub on: bool, pub temperature: i32 }
pub struct ApplicationsState { pub applications: Vec<DesktopApplication>, pub running: bool }
pub struct DesktopApplication { pub kind: String, pub id: String, pub name: String, pub subtitle: String, pub keywords: String, pub icon: String, pub exec: String, pub wmclass: String, pub categories: Vec<String> }
pub enum FontCommand { Load }
pub enum EmojiCommand { Load, StepGroup { direction: i32 }, StepTone { direction: i32 }, Remember { glyph: String }, Copy { glyph: String } }
pub enum MachineCommand { Refresh }
pub enum VersionCommand { Check, Update, ListFiles, ChangeFiles { action: VersionFileAction, paths: Vec<String> } }
pub enum VersionFileAction { Restore, Keep, Reset }
pub enum SunsetCommand { Toggle, SetTemperature { kelvin: i32 } }
pub enum ApplicationsCommand { Refresh, LaunchDesktop { desktop_id: String } }
~~~

**FontService.** Estado inicial `sans=[]`, `mono=[]`, `loaded=false`. `load()` é lazy e no-op quando loaded. Saída vazia é ignorada; JSON válido substitui listas por `sans ?? []` e `mono ?? []` e marca loaded. JSON inválido só avisa; outra chamada pode tentar novamente. `fonts.py` roda `fc-list` (timeout 10s), usa primeira alias como canonical, deduplica e ordena case-insensitive; `sans` significa todos menos mono, não uma classificação real sans-serif. Ausência/erro no helper vira listas vazias, que o adapter considera carregadas. Consumidores: `AppearanceSection` e `FontPicker`. Evidência: `services/FontService.qml`, `scripts/fonts.py`, `settings/AppearanceSection.qml`, `settings/FontPicker.qml`.

**EmojiService.** `language` e `tone` são settings; catálogo é lazy. `fresh` impede repetição após marcação de carga; mudança de idioma o zera. Launcher abre/recarrega conforme lifecycle rastreado na matriz. Recentes persistem em `${stateDirectory}/emoji-recent.json`, MRU deduplicado, máximo 24, com glyph base; tom global aplicado a variante exibida/copied. `copy` chama `wl-copy` e registra o glyph base. Python lê emoji-test, CLDR annotations e iso-codes; inclui fully-qualified, exclui Component, mantém grupos/ordem, fallback de idioma e keywords sem acentos; só emite variantes se as cinco forem encontradas. Ausência de fonte produz lista vazia + stderr; parse inválido no adapter só avisa e mantém estado anterior. Não há erro público, e `fresh=true` não prova sucesso. IDs/labels de grupo e shape persistido seguem literalmente os arquivos originais. Consumidores: `LauncherService`, `LauncherPanel`. Evidência: `services/EmojiService.qml`, `scripts/emoji.py`, `services/LauncherService.qml`, `bar/island/LauncherPanel.qml`.

**MachineService.** `refresh()` ignora enquanto processo roda; stdout vazio é ignorado. JSON válido mapeia campos textuais ausentes/null para `""`, packages não numérico para null e define `ready=true`; parse inválido só avisa e mantém estado anterior. O JSON bruto do helper pode conter null para ausência; isso difere do estado exposto pelo adapter, que colapsa os cinco campos textuais para string vazia. Helper lê PRETTY_NAME, `os.uname`, hostname, `XDG_CURRENT_DESKTOP` (com tentativa de `niri --version` quando Niri), shell do passwd e conta diretórios `/var/lib/pacman/local`; ausências retornam null. `wm` e fallback têm o significado literal do helper. Consumidores: `VelaturaBlock`, `ImpastoBlock`. Evidência: `services/MachineService.qml`, `scripts/machine.py`, `bar/island/controls/{VelaturaBlock,ImpastoBlock}.qml`.

**VersionService.** FileViews observam version e `home.manifest`; manifest change limpa `changing` e chama listagem. `check` no-op se repo vazio ou já checking; ao sair sempre limpa checking. JSON válido define answered/available; resposta indisponível limpa branch, counts, target e commits, sem resetar necessariamente os outros campos. JSON inválido só avisa e mantém estado. Cache é stale quando nunca respondeu, versão mudou ou idade excedeu 30min. `version.py check` faz `git fetch --quiet --tags` (timeout 25s, sem prompt; altera refs locais), status de branch/upstream/ahead/behind e até 12 commits; offline usa refs anteriores. Não faz checkout/merge. `update()` abre terminal para `setup update`; `ListFiles` usa `setup files list`; ações restore/keep/reset são detached. `changing` termina por watcher do manifest ou timer de 5s, sem receipt do efeito. Script possui `status` sem consumidor observado. Consumidor principal: `SystemSection`; `FaceService` também lê repo. Evidência: `services/VersionService.qml`, `scripts/version.py`, `settings/SystemSection.qml`, `services/FaceService.qml`.

**SunsetService.** Probe de `wlsunset`; stdout não vazio define available. `on` deriva de `filter.running`. Filtro executa `wlsunset -T temp -t temp` se setting ativo, disponível e não reiniciando. Temperatura é arredondada a 100K; alteração ligada faz restart após 250ms. Saída do filtro, salvo setting desligada/restart pendente, desliga `nightLight` e solicita OSD “Night light stopped”. Probe/spawn/exit não fornece causa pública detalhada. Presets e propriedades derivadas são consumidos por ControlsService, DynamicIsland e NightLightDetail; startup toca `available`. Evidência: `services/SunsetService.qml`, `services/ControlsService.qml`, `bar/widgets/DynamicIsland.qml`, `bar/island/NightLightDetail.qml`, `shell.qml`.

**ApplicationsService.** Indexer inicia 300ms após criação; `refresh()` marca running e inicia sem guard concorrente. stdout vazio é ignorado; JSON válido troca catálogo; parse inválido só avisa; sem loaded/error público. `launchDesktop` vazio é no-op; id válido chama `gtk-launch` detached sem receipt. Script usa XDG_DATA_HOME/DIRS, primeira precedência de basename válida, leitura apenas de diretórios diretos; filtra Type, Hidden/NoDisplay e Name. Payload literal tem `kind="app"`, id como basename `.desktop`, `keywords` como uma string lowercase formada por `" ".join([Name, Keywords, GenericName])`, e `subtitle` como `Comment.strip() or GenericName` (cujo default `Application` só se aplica quando a chave não existe), além de icon/exec/wmclass/categories. Não rebatizar `wmclass` nem dividir keywords em tokens. Diretório ilegível gera stderr e segue. Não se presume locale resolution ou Exec parsing. Consumidores: LauncherService/LauncherPanel e DockService. Evidência: `services/ApplicationsService.qml`, `scripts/applications.py`, `services/LauncherService.qml`, `bar/island/LauncherPanel.qml`, `services/DockService.qml`.

**Limites desta especificação.** Os adapters não publicam várias causas de falha, exit status ou receipts terminais; não sintetizar sucesso/falha para a UI. Emoji persistence malformed/missing recovery e versão dos dados Unicode dependem do JsonAdapter/dados instalados; desempate da ordenação de aplicações segue parser/origem; `Version.ChangeFiles` não confirma efeito e fetch altera refs; Sunset não distingue indisponibilidade de falha de processo; Applications admite refresh sobreposto. Fixtures sintéticas limitadas a payloads de script com schema confirmado: fonts, machine e applications (`docs/fixtures/{fonts.index,machine.report,applications.index}.json`). Não há fixtures de lifecycle/erro para esses serviços nem fixtures oficiais para Emoji, Version e Sunset, pois isso exigiria presumir estados que o original não expõe. Contratos descrevem a superfície observável, mas os limites de recuperabilidade e estado de erro são pendências reais para portabilidade 1:1.

### 12.3 Contratos tipados — Niri, Settings, Audio, Network, Bluetooth, Battery e Brightness

Escopo da rodada 5. A auditoria-base está em `BACKEND_AUDIT.md` e `INTEGRATION_AUDIT.md`; a matriz abaixo liga cada consumidor ao estado e comando/evento. Nomes de comandos/eventos Rust são uma representação proposta, não nomes de tipos QML. Payloads de runtime Quickshell (Pipewire, BlueZ, UPower) não são expostos: os tipos Rust abaixo são projeções dos campos que o código original lê. Não há validação runtime nesta rodada.

~~~rust
pub struct NiriServiceState {
    pub last_error: String,
    pub focused_workspace_id: Option<u64>, pub active_workspace_id: Option<u64>,
    pub focused_address: String, pub clients: Vec<NiriClient>, pub occupied_ids: Vec<u32>,
    pub occupied_workspace_ids: Vec<u64>, pub action_queue: Vec<Vec<String>>,
    pub active_id: u32,
    pub maximum: u32, pub span: u32,
}
pub struct WorkspaceSnapshot {
    pub id: u64, pub idx: u32, pub name: Option<String>, pub output: Option<String>,
    pub is_focused: bool, pub is_active: bool, pub is_urgent: Option<bool>,
    pub active_window_id: Option<u64>, pub raw: JsonObject,
}
pub struct WindowSnapshot {
    pub id: u64, pub workspace_id: Option<u64>, pub app_id: Option<String>,
    pub title: Option<String>, pub is_focused: bool, pub is_urgent: Option<bool>,
    pub layout: Option<JsonValue>, pub raw: JsonObject,
}
pub struct NiriClient { pub window: WindowSnapshot, pub address: String, pub class: String, pub mapped: bool, pub workspace: Option<NiriClientWorkspace> }
pub struct NiriClientWorkspace { pub id: u32, pub name: String, pub output: String }
pub type JsonObject = BTreeMap<String, SettingValue>;
pub type JsonValue = SettingValue;
pub struct NiriEvent { pub kind: String, pub data: JsonValue }
pub enum NiriCommand {
    RunAction { argv: Vec<String> }, RunActions { actions: Vec<Vec<String>> },
    FocusWorkspace { index: u32 }, FocusWorkspaceOn { index: u32, output: String },
    FocusWindow { id: u64 }, CloseWindow { id: u64 }, ToggleFloating { id: u64 },
}
pub struct SettingsState {
    pub arrived: bool, pub fresh: bool, pub state_directory: String,
    pub values: BTreeMap<String, SettingValue>, pub machine_keys: Vec<String>,
}
pub enum SettingsCommand {
    Set { key: String, value: SettingValue }, ResetProfile,
    AdoptProfile { values: JsonObject },
    SetBarZone { side: String, items: Vec<BarLayoutInput> },
    SetBeside { id: String, enabled: bool },
    SetLauncherPrefix { id: String, sigil: String },
}
pub struct BarLayoutInput { pub id: String, pub shape: Option<String>, pub figure: Option<String>, pub when: Option<String> }
pub struct AudioState {
    pub ready: bool, pub sink: Option<AudioEndpoint>, pub source_ready: bool,
    pub source: Option<AudioEndpoint>, pub volume: i32, pub muted: bool, pub icon: String,
    pub source_volume: i32, pub source_muted: bool, pub source_icon: String,
    pub outputs: Vec<AudioEndpoint>, pub inputs: Vec<AudioEndpoint>,
    pub streams: Vec<AudioStream>, pub input_streams: Vec<AudioStream>,
    pub captures: Vec<AudioStream>, pub output_prefix: String, pub card_rows: u32,
    pub panel_width: u32, pub panel_height: u32, pub microphone_height: u32,
}
pub struct AudioObjectHandle { /* identidade privada do provider; nunca o objeto PipeWire */ }
pub struct AudioEndpoint { pub handle: AudioObjectHandle, pub name: String, pub description: String, pub nickname: String, pub percent: i32, pub muted: bool }
pub struct AudioStream { pub handle: AudioObjectHandle, pub name: String, pub icon_name: String, pub percent: i32, pub muted: bool }
pub enum AudioCommand {
    SetVolume { percent: i32 }, ToggleMute, SetSourceVolume { percent: i32 }, ToggleSourceMute,
    ChooseOutput { handle: AudioObjectHandle }, ChooseInput { handle: AudioObjectHandle },
    SetStreamVolume { handle: AudioObjectHandle, percent: i32 }, ToggleStreamMute { handle: AudioObjectHandle },
}
pub struct NetworkState {
    pub devices: Vec<NetworkDeviceSnapshot>, pub wifi_enabled: bool, pub wifi_connected: bool,
    pub wired_connected: bool, pub online: bool, pub connection_name: String,
    pub state_line: String, pub strength: f64, pub icon: String, pub radio_on: bool,
    pub networks: Vec<WifiNetwork>, pub scanning: bool, pub busy_ssid: String,
    pub last_error: String, pub pending_action: String,
}
pub struct NetworkDeviceSnapshot { pub kind: String, pub connected: bool, pub wifi_networks: Vec<WifiLink> }
pub struct WifiLink { pub connected: bool, pub signal_strength: Option<f64>, pub name: String }
pub struct WifiNetwork { pub ssid: String, pub signal: i32, pub secure: bool, pub active: bool, pub known: bool }
pub enum NetworkCommand {
    Refresh, ToggleWifi, SetWifi { on: bool }, Scan { rescan: bool },
    Connect { ssid: String, password: Option<SecretInput> }, Disconnect { ssid: String }, Forget { ssid: String },
}
pub struct NetworkConnectionFinished { pub ssid: String, pub success: bool }
pub struct BluetoothState {
    pub available: bool, pub enabled: bool, pub discovering: bool, pub all_devices: Vec<BluetoothDevice>,
    pub connected_devices: Vec<BluetoothDevice>, pub devices: Vec<BluetoothDevice>,
    pub unnamed_devices: Vec<BluetoothDevice>, pub unnamed_count: u32, pub summary: String, pub icon: String,
}
pub struct BluetoothDevice { pub address: String, pub name: String, pub device_name: String, pub icon: String, pub paired: bool, pub connected: bool, pub pairing: bool }
pub enum BluetoothCommand { Toggle, SetDiscovering { on: bool }, ToggleDeviceConnection { address: String } }
pub struct BatteryState {
    pub available: bool, pub percent: i32, pub state_code: i32, pub charging: bool, pub full: bool,
    pub low: bool, pub seconds_to_empty: i32, pub seconds_to_full: i32, pub estimate: String,
    pub watts: f64, pub energy: f64, pub energy_capacity: f64, pub health_known: bool,
    pub health: i32, pub state_word: String, pub icon: String, pub tint: BatteryTint,
    pub level_icons: [String; 11],
}
pub enum BatteryTint { Dim, Good, Bad, Warn }
pub struct BrightnessState { pub displays: Vec<BrightnessDisplay>, pub dimmable: Vec<String>, pub current: Option<String>, pub available: bool, pub percent: i32, pub icon: String, pub several: bool, pub backlight_percent: i32, pub panel_width: u32, pub panel_height: u32 }
pub struct BrightnessDisplay { pub name: String, pub title: String, pub panel: bool, pub backlight: bool, pub available: bool, pub percent: i32, pub icon: String, pub level: i32, pub ceiling: i32, pub answering: bool }
pub enum BrightnessCommand { SetCurrent { percent: i32 }, StepCurrent { delta: i32 }, SetDisplay { screen_name: String, percent: i32 }, StepDisplay { screen_name: String, delta: i32 }, Refresh }
~~~

#### NiriService

- **Dados:** workspaces/windows começam vazios; foco IDs e referências iniciam null; `connected=false`, `lastError=""`, fila vazia. `connected` fica em `CompositorSnapshot`, os outros campos específicos em `NiriServiceState`; lista/foco/output comuns não são duplicados em DomainSnapshot. Workspace/window são objetos do Niri: o contrato mantém projeção dos campos efetivamente lidos e `raw` para campos adicionais, sem descartar dados originais. `clients` é shallow copy de cada janela com aliases `address=String(id)`, `class=app_id ?? ""`, `mapped=true`, e `workspace` com `id=idx`, `name ?? ""`, `output ?? ""`. `focusedAddress`, `occupiedIds`, `occupiedWorkspaceIds`, `activeOn`, `isOccupiedOn`, `isOccupied`, `isVisibleOn`, `windowsOn`, `clientsOn`, `activeId`, `maximum` e `span` mantêm suas distinções e fórmulas do adapter. `maximum` vem de `SettingsService.workspaceMax`; `activeId` fallback é 1, `activeWorkspaceId` fallback protocolar é 0.
- **Eventos/lifecycle:** inicia `niri msg --json event-stream` no singleton; cada linha é JSON. Primeiro `{Ok:{Handled:null}}` marca connected e limpa erro; `{Err:...}` desconecta e guarda JSON serializado. Eventos de workspace/janela substituem ou atualizam arrays conforme o handler QML; `WindowOpenedOrChanged` emite `windowOpened` somente em novo ID; `WindowClosed` emite sempre ID; todos eventos válidos, inclusive tipos não tratados, são encaminhados por `eventReceived(kind,data)`. JSON inválido define lastError, mantém connected/estado e não emite eventReceived. Saída não-zero marca desconectado, define mensagem `Niri event stream exited (<code>)` e tenta reconectar após 3000ms; saída zero marca desconectado e **não agenda reconnect**. Não limpar estado de janelas/workspaces ao desconectar; evento válido posterior define connected=true, mas não limpa necessariamente lastError.
- **Comandos:** `runAction`/`runActions` aceitam arrays; só arrays não vazios são enfileirados, cada arg convertido a string. Um processo `niri msg action <argv...>` roda por vez; falha só gera warning, segue fila e não devolve receipt/erro. `focusWorkspace(index)` usa output focado; `focusWorkspaceOn` quando output diferente enfileira `focus-monitor <output>` seguido de `focus-workspace <index>`; index ≤0 é no-op. Foco/fechamento/float mapeiam respectivamente `focus-window --id`, `close-window --id`, `toggle-window-floating --id`; id vazio é no-op.
- **Consumidores:** `WorkspacesWidget`, `OverviewPanel`, `IslandRest`, `DynamicIsland`, `LauncherService/LauncherPanel`, `DockService/DockMenu`, `MonitorService`, `BrightnessService`, `CaptureService`, `RecorderService`, `SessionService`, `ScreenTimeService`, `shell.qml` e Settings Bar/Windows.
- **Sem persistência.** Não traduzir todos os eventos Niri em eventos inventados: preservar kind/data para os não projetados e ordem do stream. Fonte: `services/NiriService.qml`, `services/MonitorService.qml`, `services/DesktopService.qml`, `Commands.qml`, componentes citados; auditoria `BACKEND_AUDIT.md` §Niri.

#### SettingsService

- **Estado público:** aliases do `Store`, `arrived`, `fresh`, `stateDirectory`; catálogos/constantes `launcherPrefixDefaults`, `clockFormats`, `barStyles`, `notched`, `barDefaults`, `chipShapes`, `workspaceStyles`, `chipFigures`, `besideDefaults`, `besideChoices`; funções `launcherPrefix`, `barItems`, `barZone`, `onBar`, `tidy`, `setBarZone`, `welcomeActivities`, `beside`, `setBeside`, `set`, `shapeOf`, `accepts`, `copy`, `complete`, `snapshot`, `adopt`, `reset`, `machineLayout`. Os valores persistidos são todas as propriedades declaradas por `Store` em `services/SettingsService.qml`; não são JSON livre nem somente as propriedades visualmente usadas. `machineKeys` literal: displays, lidPolicy, workspaceMode, userName, userAvatar, language, keyboard, weatherPlace, githubUser, markets, emojiTone, doNotDisturb, nightLight, nightTemperature, barHidden, desktopHidden, gameMode, cameraEnabled, cameraEffect, cameraReactions, recorderAudio, captureShape, captureKind. `profileKeys = storedKeys - machineKeys`.
- **Store JSON: 108 chaves exatas:** `clockFormat`, `clockShowsDate`, `clockShowsSeconds`, `barHeight`, `barMargin`, `islandAttached`, `barSideMargin`, `barEverywhere`, `barHidden`, `screenCorners`, `gameMode`, `windowShadow`, `barShadow`, `dockShadow`, `widgetShadow`, `windowGlass`, `surfaceStyle`, `desktopGround`, `barSides`, `wallpaperTransition`, `wallpaperDir`, `matugenEnabled`, `greeting`, `barStyle`, `barLeft`, `barRight`, `islandSummary`, `lyrics`, `islandActivities`, `islandActivitiesSeen`, `chipShape`, `chipFigure`, `launcherResults`, `launcherOrder`, `launcherFits`, `emojiTone`, `cameraEnabled`, `cameraEffect`, `cameraReactions`, `clipboardHistory`, `clipboardKeep`, `clipboardImages`, `clipboardWipeOnLock`, `lockBlur`, `lockClock`, `lockMusic`, `lockMusicGround`, `userName`, `userAvatar`, `doNotDisturb`, `recorderAudio`, `captureShape`, `captureKind`, `notesHandwriting`, `deckOnEmpty`, `spectrumOnEmpty`, `workspaceCount`, `workspaceMax`, `workspaceMode`, `workspaceStyle`, `motionScale`, `motionCurve`, `animationPreset`, `fontFamily`, `fontMono`, `notificationTimeout`, `compositor`, `keyboard`, `displays`, `keys`, `launcherPrefixes`, `desktopWidgets`, `desktopTheme`, `desktopOpacity`, `desktopHidden`, `centreButtons`, `centreTop`, `centreColumns`, `centreRows`, `centreBlocks`, `centreToggles`, `dockEnabled`, `dockPinned`, `dockEdge`, `dockAlignment`, `dockIconSize`, `dockMagnify`, `dockOpacity`, `dockGround`, `terminalGround`, `dockRunning`, `dockEverywhere`, `dockAutohide`, `dockLauncher`, `weatherPlace`, `githubUser`, `markets`, `petStyle`, `idleLock`, `idleScreen`, `idleSuspend`, `nightLight`, `nightTemperature`, `lidPolicy`, `cursorColor`, `cursorSize`, `shakeToFind`, `language`. `islandActivitiesSeen` faz parte do arquivo e dos profileKeys, embora não tenha alias público próprio.
- **Persistência e inicialização:** JSON `$XDG_STATE_HOME/quickshell/settings.json`, fallback `$HOME/.local/state/quickshell/settings.json`; XDG_STATE_HOME vazio também usa fallback. JsonAdapter Store mantém defaults no QML; não há `schemaVersion` no formato original. `arrived=false` até load; só FileNotFound habilita `fresh=true`, lê `00-keyboard.conf` para `keyboard` e grava defaults. `set` desconhecida ou antes de arrived é warning + no-op, writes não são enfileiradas. `onAdapterUpdated` agenda saver 0ms para coalescer alterações do turno. watchChanges recarrega mudanças externas.
- **Versão do contrato:** `AppSnapshot.settings.schema_version`/`revision` são metadados do protocolo Rust, não propriedades nem campos do settings.json original. Não serializar um `schemaVersion` no arquivo original como se existisse; compatibilidade/migrações seguem as keys e marcadores literais acima.
- **Migrações de leitura:** `frosted`→`glass` em surfaceStyle/desktopGround/dockGround; ground vazio herda surfaceStyle; `dockShadow` ausente no texto JSON copia barShadow; `barFullWidth:true` com `barStyle:"island"` migra para `bezel`. Essas condições e ordem precisam permanecer literais. `complete(values)` considera somente profileKeys, aceita somente shape compatível com default (default null permite null/list/map; números precisam finitos), completa ausentes com defaults; `snapshot()` contém só profileKeys; `adopt()` falha sem arrived, senão aplica perfil completo; `reset()` adota vazio e conserva machineKeys. `machineLayout()` extrai somente XkbLayout e opções `grp:*` do arquivo Xorg legado.
- **Operações auxiliares:** `setBarZone(side, items)` chama tidy (remove splits nas pontas e adjacentes), serializa bare id quando sem override e omite campos vazios; `side=="left"` grava barLeft e qualquer outro valor escolhe barRight. `setLauncherPrefix(id,sigil)` remove override se vazio ou igual default; setter aceita valor arbitrário, mas getter usa somente string de um caractere e senão devolve default. `setBeside` altera lista e atualiza `islandActivitiesSeen`; `welcomeActivities` inclui escolhas default novas uma vez. Não deduplicar itens do bar nem materializar valores herdados.
- **Consumidores:** todas as 14 seções de Settings e SettingsWindow/Panel; `shell.qml`, Bar/BarReserve/BarZone/BarChip, Island/DynamicIsland, WorkspacesWidget, Controls/LayoutEditor, Launcher, Dock, Desktop/Deck/Inspector, Lock, Theme, Profile, Capture/Recorder e vários services conforme referências em `UI_COMPONENT_CATALOG.md`.
- **Pendente concreto:** QML não define chave de versão. `complete/adopt` descarta chaves que não pertencem a profileKeys; comportamento de JsonAdapter ao carregar e regravar propriedades JSON desconhecidas precisa ser confirmado na documentação/runtime do Quickshell antes de afirmar preservação/perda. O princípio geral de preservar unknown keys no contrato Rust não prova que o original o faça. Não criar migração Rust que altere esse resultado sem decisão registrada. Fonte: `services/SettingsService.qml`, auditoria `BACKEND_AUDIT.md` §Settings.

#### AudioService

- **Dados:** sink/source default e `ready`/`sourceReady`; volume e sourceVolume em porcentagem arredondada; muted/sourceMuted; ícones literais; outputs são sinks que não são streams com audio, ordenados primeiro não-digital e depois descrição/nome locale/numeric; streams são `AudioOutStream`; inputs são fontes `Audio/Source` sem sink/stream; inStreams `AudioInStream`; captures exige ready e exclui `stream.capture.sink=="true"`, `stream.monitor=="true"`, e `application.name=="cava"`. Não-classificados ainda podem aparecer como captures antes de propriedades chegarem? O QML filtra apenas depois `ready`, preservando essa condição.
- **Atualização:** PipeWire reativo via `PwObjectTracker`; sem polling ou persistência própria. Default objects podem ser null/unready; ausência dá ready=false, volume 0 e muted=false. `outputPrefix` é a sequência comum de palavras, nunca considera última palavra; `outputName` remove esse prefixo e reescreve `HDMI / DisplayPort N Output` como `HDMI N`. streamName escolhe application.name→process.binary→node.name e capitaliza só o primeiro caractere; streamIcon escolhe icon-name→binary, lowercase, Theme.appIcon.
- **Comandos:** percent clamp 0..100 e converte /100 para PipeWire; output/input selection escreve `preferredDefaultAudioSink/Source`; toggles negam valor atual; stream/source volume só se objeto/ready disponível. Não há receipt do setter nem confirmação além da propriedade reativa.
- **Consumidores:** VolumeModule, SoundDetail, MicrophoneDetail, ModuleService, controls/BlockFace e cards, OSD (debounce/suppress), Wides/Squares, faces analogue/sticker; PRIVACY consome AudioService inputs/captures.
- **Pendente:** contratos precisam associar handle de stream/node ao objeto reativo e expiração quando node desaparece; IDs/capabilities PipeWire devem ser definidos pelo provider, não runtime object no frontend. Fonte: `services/AudioService.qml`, `services/PrivacyService.qml`, componentes citados; `BACKEND_AUDIT.md` §Audio.

#### NetworkService

- **Dados:** NetworkManager reativo fornece devices, wifiDevice/wiredDevice, wifi/wired connection e `Networking.connectivity===Full` para online. `radioOn` vem de `nmcli -t -f WIFI radio all`, comparado literalmente a `enabled`. `connectionName`: Wired > nome SSID ativo > Off se rádio desligado > Not connected se há wifiDevice > Unavailable. `strength` é 0 fora do Wi-Fi conectado; signalStrength >1 divide por 100. `stateLine`: Ethernet · online/no internet, Wi-Fi · online/no internet, Wi-Fi radio off, ou Online/Nothing reaches the internet, conforme precedência original. Icons: wired 󰈁; radio off 󰤮; wifi connected 󰤨; radio on/disconnected 󰤯. `strengthIcon` thresholds >=75, >=50, >=25, else, glyphs 󰤨/󰤥/󰤢/󰤟.
- **Listagem/eventos:** scan(rescan=false) inicia `network.py list`; true usa `rescan`, que chama `nmcli device wifi rescan` e lista logo depois sem aguardar scan completar. Sem rádio, list ainda roda e normalmente lista vazia. Radio vindo para on agenda scan após 1200ms; radio off limpa networks. stdout vazio preserva lista antiga; parse inválido warning; onExited limpa scanning. network.py retorna `[{ssid,signal,secure,active,known}]`, exclui SSID vazio, decodifica `\:` por parser terse, junta sightings por SSID (maior sinal, OR active/secure, `known` da lista de conexões wireless) e ordena active primeiro/depois sinal desc.
- **Comandos:** Refresh relê rádio; ToggleWifi alterna estado observado; SetWifi(on) fixa estado; Scan(rescan); Connect(ssid,password default vazio), Disconnect(ssid), Forget(ssid). connectProcess é compartilhado: cada ação seta pendingAction/busySsid, limpa lastError; final calcula success por exitCode==0, libera busy, sucesso limpa erro, falha de connect sem stderr usa fallback inglês, emite connectionFinished somente para connect; depois limpa pendingAction, refresh e scan(false). stderr do helper é sanitizado em português e preservado como lastError; detalhes nmcli/password não devem vazar. Script timeouts 25s padrão, 60s connect; sem nmcli/exception retorna None e stderr. Nenhum password persiste em Settings/NetworkManager passa a deter perfil.
- **Segurança/paridade:** original passa password como argv para network.py. Não repetir exposição em `/proc`; transportar o mesmo segredo por canal protegido sem mudar Connect/resultado observável. Desconexão e forget não emitem connectionFinished; erro de list/rescan não ganha novo estado público.
- **Consumidores:** NetworkModule, NetworkDetail, TogglesBlock (refresh), ModuleService, Wides/Squares e faces analogue/sticker. `SystemService` também mantém um provider separado via `scripts/system.py` para o bloco de toggles; não substituir esse caminho por NetworkService implicitamente. Fonte `services/NetworkService.qml`, `scripts/network.py`, UI citada; `BACKEND_AUDIT.md` §Network e `INTEGRATION_AUDIT.md` network.py.

#### BluetoothService

- **Dados/reatividade:** adapter é `Bluetooth.defaultAdapter`; absent => available=false, enabled=false, discovering=false e arrays vazios; todos os `adapter.devices.values` entram em allDevices. `deviceName` define named (não `name`, que pode fallback para MAC). Lista principal inclui named OU paired OU connected OU pairing, ordenada connected primeiro, paired depois, `name.localeCompare`; unnamed é complemento, ordenado por address. Não remover pareados mesmo sem nome anunciado. summary é Unavailable/Off/No devices/nome único/N devices; icon disabled ou unavailable 󰂲, enabled+connected 󰂱, senão 󰂯. `deviceIcon` mapeia audio-headset/headphones→󰋋, audio-card→󰓃, input-mouse→󰍽, input-keyboard→󰌌, phone→󰄜, computer→󰟀, fallback→󰂯. BlueZ fornece updates reativos; pareamento persistido pertence ao BlueZ, sem settings.
- **Comandos:** Toggle alterna `adapter.enabled`; SetDiscovering(on) atribui adapter.discovering quando adapter existe; ToggleDeviceConnection(address) nega `device.connected`. Não existe comando Pair, remoção nem PIN agent nesta UI/service; não adicioná-los por inferência. BluetoothDetail começa discovery ao completar se enabled e o desliga ao destruir; o switch chama toggle e em seguida setDiscovering(checked), mesmo que atualização de enabled ainda não tenha chegado.
- **Consumidores:** BluetoothModule, BluetoothDetail, ModuleService, Wides/Squares e faces analogue/sticker. `SystemService` é provider paralelo para outro bloco de toggle, com polling/status próprios e `system.py`; manter essa duplicação funcional em vez de unir os estados sem evidência.
- **Pendente:** adapter/device removido no meio da ação necessita lifecycle/reconnect equivalente no provider (**runtime**); operação de toggle tem identidade observável por `address`, não deve expor objeto BlueZ. Fontes `services/BluetoothService.qml`, `services/SystemService.qml`, `bar/island/BluetoothDetail.qml`, `BACKEND_AUDIT.md` §Bluetooth.

#### BatteryService

- **Dados:** `available` somente `displayDevice != null && ready && isLaptopBattery`; sem bateria expõe percent 0, state `Unknown`, bools false, tempos/watts/energia/health 0, estimate/stateWord vazios e ícone de ausência. Percent = round(fraction*100); low <=20 sem charging/full; tint thresholds <=15 bad, <=35 warn, else good; glyph table explícito 0..10. Estimativa: full→`Fully charged`; tempo <=0 retorna `Charging` quando carregando ou vazio descarregando; caso contrário `h h m min` quando horas >0 ou `m min`, sufixo `to full`/`left`. watts abs(changeRate); health somente se suportado, arredondado.
- **Atualização:** propriedades UPower reativas/D-Bus; sem polling, comando ou persistência. State bruto é enum UPower; mapping numérico local deve ser resolvido pelo binding, UI usa `charging/full` e strings derivadas. Ordem exata de `level_icons` (percent steps 0 a 10): `󰂎`, `󰁺`, `󰁻`, `󰁼`, `󰁽`, `󰁾`, `󰁿`, `󰂀`, `󰂁`, `󰂂`, `󰁹`.
- **Consumidores:** BatteryModule/Widget, OSD na mudança charging, ModuleService low/tint/availability, DynamicIsland, Control Center/System block, LockSurface, Wides/Squares e faces analogue/sticker.
- **Pendente:** enum numérico UPower e tratamento de `ready`/hot-swap exigem binding/reconnect equivalente; nada de fixture com enum numérico oficial sem map confirmado. Fonte `services/BatteryService.qml`, `OsdService.qml`, consumidores; `BACKEND_AUDIT.md` §Battery.

#### BrightnessService

- **Dados:** uma display state por Quickshell screen; painel reconhecido por `^(eDP|LVDS|DSI)-`; title Built-in ou model/name. Primeiro `/sys/class/backlight` (ordenação ls) lido uma vez; FileView observa raw brightness; max; backlight é painel com max>0 ou output único sem DDC bus. Externos descobertos apenas se houver screen não painel com `ddcutil detect --brief`; map connector Niri name→I2C bus. display answering exige resposta `VCP 10 C level maximum`; disponível se backlight ou bus + answering. Percent de backlight round(raw/max*100), DDC level percentual do VCP. `current` = display do focused output se dimmable, senão primeiro dimmable; `several` iff >1; icon thresholds <34, <67, caso contrário. `adjusted(display)` apenas na mudança sysfs raw ou ajuste pedido (inclusive tecla no limite); leitura DDC não emite.
- **Comandos/lifecycle:** SetCurrent(percent), StepCurrent(delta), Refresh (chama `read()` em cada display, mas somente DDC com bus lê; sysfs já watched). `setPercent` ignora unavailable, clamp 1..100, não repete valor desejado/current; `step` parte de wanted se pendente, e em limites emite adjusted sem write. Uma escrita por monitor; wanted mais recente coalesce enquanto reader/writer; DDC escala percent para ceiling VCP10, flags originais preservadas; falha write DDC não retry, relê; sem resposta tira display de dimmable. Hotplug screensChanged debounce 1500ms; sem polling contínuo. Sysfs writer `brightnessctl --quiet set N%`; falha não tem erro estruturado. DDC/screen state não é persistido.
- **Consumidores:** BrightnessModule/Detail, Controls BlockFace, DynamicIsland sizing, OSD (usa display do sinal, debounce e suprime se painel aberto), ModuleService, Wides/Squares e faces analogue/sticker; keys do sistema atualizadas no Surface externo.
- **Pendente:** identificação de display/monitor stable ID para handle de comando e publicação de falhas sysfs sem transformar estados não expostos em sucesso; DDC/sysfs permission é dependência do provider, não autoelevação. Fontes `services/BrightnessService.qml`, `OsdService.qml`, UI citada; `BACKEND_AUDIT.md` §Brightness.

**Fixtures desta rodada:** `niri.events.mock.jsonl` (handshake/eventos sintéticos com campos consumidos pelo parser), `settings.defaults.json` (108 defaults extraídos dos inicializadores QML) e `settings.partial.json` (perfil parcial para `complete()`), `audio.snapshot.json`, `network.list.json` (shape exato do helper), `bluetooth.snapshot.json`, `brightness.snapshot.json`. São sintéticas e não substituem prova runtime; não incluem senha, endereço real, caminho do perfil do usuário ou valores lidos da máquina. Sem fixture de Battery: o schema carrega o enum numérico UPower, cujo mapeamento instalado não está confirmado.

## 13. Extensão derivada da auditoria de superfícies UI (15/11/2026)

Esta seção fecha os comandos de domínio descobertos nos consumidores visuais. Continua sendo draft; nomes e payloads são propostos para revisão de Codeína, não especificação congelada.

### Desktop / widgets

`DesktopCommand` precisa representar:
- AddWidget { module_id, output_id, cell? }, RemoveWidget { key }, PlaceWidget { key, output_id, cell }, ResizeWidget { key, family }, CycleFamily { key, direction }.
- SetWidgetTheme { key, theme? }, SetWidgetOpacity { key, opacity? }, SetWidgetCaption { key, caption? }, SetWidgetPicture { key, FileGrant? }.
- SetSpectrumLook { key, look, fill, lows, peaks, colors, bar_width, gap, opacity }, MoveSpectrumToEdge { key, output_id, edge }, MoveSpectrumToGrid { key, output_id, cell }, RemoveSpectrum.
- AddDeck { output_id, edge }, MoveDeck { key, edge }, ReorderDeckNote { key, order }, TransferDeckNote { note_key, from_edge, to_edge }, ConvertDeckNoteToWidget { note_key, output_id, cell }, RemoveDeckNote { key }.
- SetGalleryPosition { output_id, position }, SetGallerySize { columns, rows }, DropWidgetToTray { key }.
- UpdateNote/ArchiveNote/RestoreNote/DeleteNote permanecem NotesCommand, mesmo quando originados de widget/Deck.
- Drag preview, selected key, pointer coordinates, inspector open e nearest-free candidate são estado transitório da UI/sessão; backend recebe apenas commit com output/cell/family. O backend valida colisões e retorna posição efetiva.

### Dock / tray

`DockCommand` precisa distinguir:
- PinApplication { desktop_id }, UnpinApplication { desktop_id }, ReorderPinned { desktop_ids }, ActivateApplication { desktop_id }, LaunchNewInstance { desktop_id }, ActivateWindow { window_id }, CloseWindow { window_id }, CloseApplication { desktop_id }.
- Menu open/hover/selected index e magnification são estado efêmero do frontend; pinned order é persistente. Tray DBusMenu commands preservam item ID + action/scroll delta, sem serializar callback.
- A lista pinned é compartilhada por Dock e Launcher; um único store/service deve ser dono da ordem.

### Bar layout / module presentation

`SettingsCommand` precisa cobrir operações específicas que o QML faz em SettingsService:
- SetBarZone { side, items }, SetBarItemLook { side, index, shape?, figure?, when? }, SetBeside { module_id, enabled }, SetLauncherPrefix { mode, sigil }, SetModulePreferences { module_id, typed_values }, SetWorkspaceStyle { style }, SetBarAppearance { style, sides, height, margins, corners, shadow }.
- `items` preserva duplicatas, ordem e split/tray/workspaces estruturais. Índice é relativo à lista do lado; API não deve deduplicar por module_id.
- `shape/figure/when = None` significa herdar configuração global; não converter None em default materializado.
- Mudança de layout não pode recriar service subscriptions enquanto o mesmo moduleId permanece; key estável da instância é requisito do frontend.

### Launcher / panels / notification actions

- LauncherCommand precisa cobrir SetQuery, SetMode, MoveSelection, ActivateResult{modifiers}, DeleteClipboardEntry e StepEmojiGroup/StepEmojiTone; modo e query podem ser estado de sessão, não settings persistentes.
- PanelCommand cobre Open/Close/Toggle, SetPage, Back/Forward, mas focus/loader state permanece na camada UI. A troca de painel deve respeitar handover da ilha e priorizar auth/notification/OSD como definido em UI_SURFACE_AUDIT.md.
- NotificationCommand inclui InvokeAction{notification_id, action_id}, Dismiss, ClearHistory; ação default só existe enquanto notification ativa. Histórico não deve expor ação executável depois de fechado.
- Timeout de notificação segue `NotificationService.timeoutFor`: critical não expira automaticamente; timeout positivo do emissor é limitado a 15000 ms; sem timeout do emissor usa `SettingsService.notificationTimeout`. A implementação Rust restaura as duas primeiras regras, mas ainda usa fallback fixo de 5000 ms até conectar Settings; não marcar paridade completa.
- `TransientEvent { icon, label, progress }` é a entrada tipada do OSD para a UI: `progress < 0` omite a barra; valores restantes são limitados a `[0, 1]`; o reducer descarta eventos com painel aberto/inactive e expira em 1800 ms. `Shell::request_osd` distribui pelos outputs e só a ilha live arbitra. Providers nativos de Audio/Brightness/Battery ainda precisam emitir esses eventos com debounce e supressão equivalentes a `OsdService.qml`.
- TrayMenu actions usam opaque DBus menu node/action IDs; nunca shell command string.

### Screen time / games / pet / controls

- ScreenTimeCommand: SelectView{day|week|month}, SelectDate, DrillIntoBucket, SelectGroup/SelectApp; report read-only segue o snapshot, filtros/selection são UI state.
- GamesCommand: Start{game_id}, Restart, Leave, SubmitScore{game_id, score}; score é persistido por GamesService e o best usa lower-is-better para jogos de movimentos. O frame não grava record diretamente.
- PetCommand: Feed, Play, Rename, BringOut{species_id}; cooldown/mood/XP/milestones são domínio persistido, não estado otimista do botão.
- ControlsCommand já listado precisa incluir ToggleTile{toggle_id}, ActivateDoor{door_id}, SetBlockLayout, MoveBlock, ResizeBlock, SetBlockContents, ReorderTopRow; grid candidate e hover size são preview, não commit.

### Limites explícitos de compatibilidade

- Não adicionar Niri keybinding editor: ShortcutService expõe 32 actions mas original não lê/edita binds. Settings apenas orienta para NiriMod.
- Não inventar window picker para Capture: `root.windowUnder()` não está definido na fonte ativa. Capture Window continua BLOCKED até confirmar implementação original.
- FileGrant obrigatório para picker de arquivos de perfil/avatar/wallpaper/capture; UI não passa paths arbitrários ao backend.
- Estado de game/pet/desktop/profile tem persistência própria; não mover scores, XP ou dados de máquina para ProfileService.
- API pública não deve expor IDs de UI/índice de Repeater como IDs de domínio, exceto índice local explicitamente usado por settings list e validado contra revision.

## 14. Extensão de snapshots/eventos para superfícies completas

A auditoria visual mostrou que o contrato v0.1 inicial precisa representar explicitamente dados consumidos por bar layout, launcher, dock, desktop e Deck. Os tipos abaixo são candidatos para a revisão de Codeína; ainda não congelar nomes nem obrigar serialização direta desses exemplos.

~~~rust
pub struct BarState {
    pub outputs: Vec<BarOutputState>,
}
pub struct BarOutputState {
    pub output_id: OutputId,
    pub live: bool,
    pub style: BarStyle,
    pub left: Vec<BarItem>,
    pub right: Vec<BarItem>,
    pub activities: Vec<ActivitySnapshot>,
    pub overflow: u32,
    pub workspace_style: WorkspaceStyle,
}
pub enum BarItem {
    Module { instance_id: String, module_id: ModuleId, look: Option<ModuleLook> },
    Split,
    Workspaces,
    Tray,
}
pub struct ModuleLook {
    pub shape: Option<String>,
    pub figure: Option<String>,
    pub when: Option<String>,
}

pub struct LauncherState {
    pub mode: LauncherMode,
    pub query: String,
    pub selected: Option<usize>,
    pub results: Vec<LauncherResult>,
    pub application_index_status: Availability,
}
pub enum LauncherMode { Applications, Calculator, Windows, Timer, Clipboard, Emoji, Desktop }
pub struct LauncherResult {
    pub id: String,
    pub kind: LauncherResultKind,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<IconRef>,
    pub enabled: bool,
    pub action: LauncherActionId, // opaque typed action, never shell text
}

pub struct DesktopState {
    pub widgets: Vec<WidgetSnapshot>,
    pub gallery: GallerySnapshot,
    pub editing: bool,
}
pub struct WidgetSnapshot {
    pub key: WidgetKey,
    pub module_id: ModuleId,
    pub output_id: OutputId,
    pub family: WidgetFamily,
    pub cell: GridCell,
    pub overrides: WidgetOverrides,
}
pub struct GridCell { pub column: u16, pub row: u16 }
pub struct WidgetOverrides {
    pub theme: Option<String>,
    pub opacity: Option<u8>,
    pub caption: Option<String>,
    pub picture: Option<FileGrant>,
    pub spectrum: Option<SpectrumLook>,
}
pub struct DeckState {
    pub edges: Vec<DeckEdgeSnapshot>,
}
pub struct DeckEdgeSnapshot {
    pub output_id: OutputId,
    pub edge: Edge,
    pub notes: Vec<DeckNoteRef>,
}
pub struct DockState {
    pub pinned: Vec<DockApplication>,
    pub running: Vec<DockApplication>,
    pub edge: DockEdge,
    pub alignment: DockAlignment,
    pub output_policy: DockOutputPolicy,
}

~~~

- BarState preserva duplicatas e IDs de instância; dois módulos iguais podem estar em posições diferentes e ter overrides diferentes. live é output state, não preferência.
- LauncherState.results é derivado de query+mode e pode ser atualizado em lote; índice selecionado é transitório e não precisa ser persistido. Emoji/clipboard são providers separados, não string matching no frontend.
- DesktopState.widgets mantém stable key e output identity; nearest-free/collision é calculado no domínio. GallerySnapshot é catálogo/capabilities, não estado de widget colocado.
- WidgetOverrides.picture é grant/referência segura; nunca serializar path arbitrário sem validação/revocation strategy. A representação real de path local e persistência precisa ser decidida.
- DeckState é uma localização alternativa do note, não cópia. Um note não pode existir simultaneamente no grid e em um edge deck.
- DockState.pinned e running são fontes diferentes, com desktop ID como identidade de app e window ID como identidade de janela; preservar pin order e agrupamento.
- UI ephemeral (hover, focus, pointer, open menu, selected index, drag preview, inspector position, animation progress) permanece no frontend; não poluir snapshot persistente com coordenadas de ponteiro.
- Eventos incrementais devem incluir BarLayoutChanged, LauncherChanged, ApplicationsChanged, DesktopChanged, DockChanged, DeckChanged e WorkspacesChanged, com revision monotônica e payload suficiente para atualizar sem reler I/O.
- AppSnapshot pode fornecer Bar/Desktop/Dock/Deck como subestados opcionais quando o service está indisponível, mas uma falha de desktop gallery não deve esconder widgets persistidos.

### Tipos auxiliares propostos

~~~rust
pub struct ModuleId(pub String);
pub struct WidgetKey(pub String);
pub struct LauncherActionId(pub String);
pub struct IconRef(pub String);

pub enum BarStyle { Grouped, Spread, Island, Bezel }
pub enum WorkspaceStyle { Dots, Bars, Rings, Numbers, Roman, Kanji, Greek }
pub enum WidgetFamily { Square2x2, Wide4x2, Large4x4, Band8x2 }
pub enum Edge { Top, Bottom, Left, Right }
pub enum DockEdge { Bottom, Left, Right }
pub enum DockAlignment { Start, Center, End }
pub enum LauncherResultKind { Application, Window, Calculation, Timer, Clipboard, Emoji, Desktop }
pub struct ActivitySnapshot {
    pub kind: ActivityKind,
    pub title: Option<String>,
    pub value: Option<String>,
    pub actionable: bool,
}
pub struct SpectrumLook {
    pub look: String,
    pub fill: String,
    pub lows: String,
    pub peaks: bool,
    pub color: String,
    pub color2: Option<String>,
    pub bar_width: f32,
    pub gap: f32,
    pub opacity: u8,
}
pub struct GallerySnapshot {
    pub available_modules: Vec<ModuleId>,
    pub supported_families: BTreeMap<ModuleId, Vec<WidgetFamily>>,
}
pub struct DeckNoteRef { pub key: String }
pub struct DockApplication {
    pub desktop_id: String,
    pub title: String,
    pub pinned: bool,
    pub windows: Vec<WindowId>,
}
pub enum DockOutputPolicy { EveryOutput, LiveOutputOnly }
~~~

Esses auxiliares são tipos candidatos, não shape final. DockOutputPolicy deve ser confirmado com o setting real; nomes de enum podem mudar na revisão. O frontend não pode inferir supported_families a partir do nome do módulo: recebe capabilities do catálogo real.

## 15. Status de prontidão após revisão técnica

Este contrato permanece draft e **não pode ser congelado nem usado como especificação completa de implementação 1:1** até fechar os bloqueadores registrados em `docs/APP_API_REVIEW.md`. O `ServiceId` contém 61 IDs únicos, cobrindo nominalmente os 61 serviços catalogados; isso não valida completude de snapshots, eventos, comandos ou lifecycle. Os tipos das seções 13–14 são propostas parciais, não APIs aprovadas.

`Capture Window` está BLOCKED. A fonte ativa contém a chamada sem definição `root.windowUnder(mouse.x, mouse.y)` em `capture/CaptureOverlay.qml`; `services/CaptureService.qml` declara apenas `region`/`screen`, registra que o Niri IPC não expõe retângulos globais e diz que um fluxo nativo separado não foi ligado. `scripts/capture.py` não implementa seleção de janela. Não há contrato de substituição: localizar a implementação original ou reproduzir e documentar o comportamento/defeito antes de decidir qualquer semântica.


## 16. Integração GDK4 de monitores e lifecycle do backend

A implementação opcional `gdk4-observer` conecta o contrato de Monitor à topologia do display sem acoplar o reducer Niri à UI:

- `GdkMonitorObserver` observa o `gio::ListModel` de `GdkDisplay::monitors()`. `items-changed` representa hotplug/remoção; após a alteração, os handlers de geometria são refeitos para a lista atual.
- Cada `GdkMonitor` ativo recebe `notify::geometry`. Hotplug e geometry chamam somente `NiriRefreshHandle::refresh_outputs()`; o serviço Niri aplica debounce de 120 ms e coalescing e executa `niri msg --json outputs` em worker.
- O observer captura connector, fabricante/modelo, geometria lógica, scale factor e refresh rate. `match_gdk_monitors` associa por connector exato; usa geometria lógica apenas se a associação for única nos inventários GDK e Niri. Fabricante/modelo não são identidade. Match ambíguo, output disabled ou removido fica sem associação.
- Attach/drop e callbacks GDK pertencem ao main thread/contexto da UI. O observer desconecta o handler da lista e todos os handlers de geometry no Drop.
- `ApplicationRuntime` possui `NiriSession` e `GdkMonitorObserver` pela vida útil do backend. `shutdown()` é idempotente, desconecta o observer antes de parar/juntar os workers Niri. O frontend acessa o canal de eventos pelo `session()`.

**Integração concluída nesta fatia:** `GtkApplication` mantém `ApplicationRuntime` nos callbacks existentes; `GtkEventBridge` recebe o canal único em worker bloqueante e despacha no main context via GLib. O shutdown para/junta primeiro o backend e depois junta o bridge. Este adapter não cria janelas layer-shell ou superfícies e não valida paridade visual.

O teste de integração feature-gated inicializa GTK, abre o display da sessão atual, confirma pelo menos um monitor GDK e valida attach/shutdown idempotente do runtime. Testes puros cobrem matching exato, fallback geométrico único, ambiguidade, remoção e output disabled. Não simulam remoção física nem alteração geométrica no display.

## 17. Bootstrap GTK4 e ownership do runtime (16/11/2026)

- O binário GTK4 é `src/main.rs`; o callback `activate` cria `ApplicationRuntime` uma única vez e o mantém enquanto a aplicação estiver ativa.
- O callback `shutdown` retira o runtime e chama `shutdown()`; observer GDK é desconectado antes de os workers Niri serem parados/joinados.
- A superfície temporária recebe `monitor_count` e um `NiriRefreshHandle` cloneável. O handle apenas enfileira solicitação de refresh; não expõe subprocessos, canais internos ou estado mutável.
- A janela não faz polling de estado nem I/O bloqueante. `GtkEventBridge` encaminha snapshots/eventos da sessão ao main context; a superfície continua diagnóstica e não declara paridade visual.
- A CLI de desenvolvimento não deve encaminhar `ipc` para `qs`; IPC nativo permanece pendente até existir contrato e handlers reais.
- `NiriSessionSnapshot` inclui revisão monotônica e cache consistente de Niri + outputs; revisão inicial 0 tem workspaces/janelas vazios e outputs `None`. `ApplicationRuntime::snapshot()` não realiza I/O nem duplica lógica do reducer.
- `ApplicationRuntime::take_events()` transfere o receiver de consumidor único ao bridge; chamadas `recv*` posteriores respondem desconectado.
- `ApplicationRuntime::monitor_matches()` combina a lista GDK atual com o snapshot de outputs atual. Recompute após evento de snapshot; callbacks de hotplug/geometria já atualizam o inventário GDK e solicitam refresh Niri.
