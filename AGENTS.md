# AGENTS.md

Guia para agentes de código que trabalham no dcom.

**Regra de manutenção:** qualquer mudança de estrutura, comando ou decisão atualiza este arquivo no mesmo commit ou PR. O `README.md` também fica sempre em dia, mas continua curto, em palavras fáceis e sem detalhes de arquitetura.

## Estado atual

Spike da fase 0: o cliente mostra a interface com dados de exemplo (`client/src/mock.rs`) e ainda não conecta ao Convex. O backend só tem o schema, sem funções. O que está marcado como **planejado** ainda não existe.

## O que é o dcom

App de comunicação para comunidades de até algumas centenas de pessoas: cliente desktop nativo em Rust, backend inteiro no Convex e quase nenhuma infra própria.

## Onde fica cada coisa

| Caminho | Conteúdo |
| --- | --- |
| `client/src/app.rs` | Telas e lista virtualizada de mensagens |
| `client/src/theme.rs` | Cores e fontes |
| `client/src/mock.rs` | Dados de exemplo do spike; sai quando o Convex entrar |
| `client/src/api/` | **Planejado:** camada fina de tipos sobre o crate `convex` |
| `convex/schema.ts` | Schema das oito tabelas |
| `convex/adapters/` | **Planejado:** adaptadores `storage`, `sfu`, `auth` |
| `selfhost/` | **Planejado:** `docker-compose.yml`, variáveis de ambiente e guia |

## Comandos

| Objetivo | Comando |
| --- | --- |
| Instalar dependências do backend | `npm install` |
| Lint e tipos do backend | `npm run lint` |
| Rodar o backend (exige projeto Convex configurado) | `npm run dev` |
| Rodar o cliente | `cargo run -p dcom-client` |
| Build de release | `cargo build --release -p dcom-client` |
| Testes do cliente (ainda não há testes) | `cargo test` |
| Lint do cliente | `cargo clippy --all-targets -- -D warnings` |
| Formatação do cliente | `cargo fmt --check` |
| Medir o scroll com 10 mil mensagens | `DCOM_BENCH=<arquivo> target/release/dcom` (10 s, sem vsync, grava o fps) |
| Subir uma instância self-hosted | **Planejado:** `docker compose up` em `selfhost/` |

## Design

Referência: layout de três colunas, escuro e arredondado.

- **Trilho (92 px):** logo, Conversas (DMs), servidores com nome embaixo do ícone, botão de adicionar.
- **Lista (340 px):** busca e cartões de canais ou DMs com avatar, prévia, hora e não lidas.
- **Chat:** cabeçalho, balões com avatar e mensagens agrupadas por autor, campo de escrita arredondado.

Use as cores de `theme.rs`, sem valores soltos. Em caso de dúvida, escolha o mais simples de usar.

## As quatro restrições fixas

Orientam todas as outras decisões. Não as flexibilize sem decisão explícita do dono do projeto.

1. **Extremamente leve.** O cliente é Rust nativo, sem Electron e sem webview. Cada dependência nova precisa justificar o peso que adiciona.
2. **API toda no Convex.** Dados, regras de negócio, permissões e tempo real ficam em funções Convex. O cliente não fala com nenhum outro backend de dados.
3. **Infra mínima.** Cloudflare Workers entram só quando o Convex não resolve bem um problema específico, por motivo medido. Nenhum servidor próprio é obrigatório, e nada essencial pode depender de Workers.
4. **Self-hostável por qualquer pessoa.** Todo serviço gerenciado da instância oficial precisa ter uma alternativa open source que se configura sem mudar código. O cliente conecta em qualquer instância pela URL.

Princípios derivados:

- Simplicidade de uso vence quantidade de recursos.
- Toda checagem de permissão acontece no Convex, nunca só no cliente.
- Atualizações muito frequentes (presença, "digitando...") são limitadas por tempo para não estourar o consumo de funções.

## Metas de leveza

Medidas a cada release. Os valores são iniciais, a confirmar no spike da fase 0.

| Métrica | Meta | Como medir |
| --- | --- | --- |
| RAM em repouso (1 servidor aberto) | < 100 MB | RSS do processo após 5 min parado |
| Tamanho do binário | < 25 MB | Build release com strip |
| Abertura a frio até a lista de canais | < 1 s | Do launch até o primeiro frame com dados |
| CPU em repouso | ~0% | Média em 5 min sem interação |
| Scroll do histórico | 60 fps com 10 mil mensagens | Contador de frames no build de debug |

Uma regressão acima de 10% em qualquer métrica bloqueia o release até ser explicada.

Medições do spike (2026-11-06, Windows 11, GPU AMD, ainda sem `convex` e tokio):

| Métrica | glow | wgpu |
| --- | --- | --- |
| RAM após 5 min | 156 MB (**acima da meta**) | 398 MB |
| Binário | 5,3 MB | 8,5 MB |
| CPU em repouso | 0% | 0,001% |
| Scroll com 10 mil mensagens, sem vsync | 2966 fps | 75 fps (parece limitado pelo monitor) |

O driver OpenGL da AMD (`atio6axx.dll`) ocupa 80 MB do processo. Abertura a frio ainda não foi medida, porque depende de dados reais.

## Arquitetura

- O cliente mantém uma única conexão WebSocket com o Convex para dados e tempo real.
- `subscribe` do crate `convex` devolve um stream que atualiza a cada mudança no servidor. A thread do tokio chama `request_repaint` quando uma assinatura muda; sem eventos, o egui não redesenha.
- Auth, anexos e SFU ficam atrás de adaptadores no Convex, escolhidos por variável de ambiente. A instância oficial e uma self-hosted rodam o mesmo código.
- As credenciais do SFU ficam no backend, que decide quem pode publicar ou ouvir. Uma action do Convex chama a API do SFU, então voz não exige Worker. O cliente fala com uma camada de sinalização própria, nunca com a API de um fornecedor.

| Camada | Instância oficial | Self-host |
| --- | --- | --- |
| UI desktop | egui via eframe | Igual |
| Cliente de dados | crate `convex` sobre tokio | Igual, apontando para a URL da instância |
| Backend | Convex Cloud | Convex self-hosted (Docker, SQLite ou Postgres) |
| Autenticação | OIDC com JWT validado pelo Convex | Provedor OIDC configurável ou auth embutida no Convex |
| Anexos | Convex file storage; R2 se a banda pesar | Convex file storage ou storage compatível com S3 |
| Voz (fase 3) | WebRTC + Cloudflare Realtime SFU | SFU open source (ex.: LiveKit) |
| Áudio no cliente (fase 3) | cpal + Opus | Igual |
| Workers | Opcionais, só por motivo medido | Não necessários |

## Escopo do MVP (v0.1)

Chat de texto completo, sem voz, na instância oficial e em self-host.

- **Conta e instância:** escolha de instância (oficial por padrão, ou URL própria); login pelo navegador do sistema com token no keychain do SO; perfil com nome de exibição e avatar.
- **Servidores e canais:** criar, renomear, trocar ícone e excluir servidor; criar, renomear, reordenar e excluir canais de texto; convite por link ou código com expiração e limite de usos; cargos fixos (dono, admin, membro); expulsar e banir.
- **Mensagens:** enviar, editar e apagar; histórico paginado com scroll infinito; menções; não lidas por canal e servidor; DMs 1-a-1; notificações nativas para menções e DMs.
- **Tempo real:** mensagens ao vivo via assinatura de query; presença com heartbeat limitado; "digitando..." com throttle.
- **Self-host:** um `docker compose up` sobe o backend com as funções do projeto; guia com variáveis de ambiente documentadas.

**Fora do escopo por enquanto:** voz e vídeo (fase 3), compartilhamento de tela, mobile e web, criptografia ponta a ponta, federação, bots, webhooks e API pública, cargos customizados, threads, fóruns e eventos, cache offline persistente.

## Modelo de dados inicial

DMs reaproveitam `channels` para não duplicar a lógica de mensagens.

| Tabela | Campos principais | Índices |
| --- | --- | --- |
| `users` | tokenIdentifier, displayName, avatarId, lastSeenAt | por tokenIdentifier |
| `servers` | name, iconId, ownerId | por ownerId |
| `members` | serverId, userId, role, joinedAt | por serverId; por userId; por serverId + userId |
| `channels` | serverId (opcional), name, kind (`text` ou `dm`), position | por serverId + position |
| `channelParticipants` | channelId, userId (só para DMs) | por userId; por channelId |
| `messages` | channelId, authorId, body, mentions, editedAt, deletedAt | por channelId + `_creationTime` |
| `readStates` | userId, channelId, lastReadMessageId | por userId + channelId |
| `invites` | code, serverId, createdBy, expiresAt, maxUses, uses | por code |

Presença e "digitando..." ficam em uma tabela efêmera com expiração, fora das tabelas principais, para não disparar invalidações nas queries de mensagens.

## Convenções

**Código**

- Toda função Convex pública valida argumentos e checa autenticação e permissão no servidor.
- Serviço externo só é acessado pelo seu adaptador em `convex/adapters/`, nunca direto de uma função de negócio.
- Configuração de instância vem de variável de ambiente; nunca fixe URLs ou credenciais no código.
- Ao adicionar uma dependência ao cliente, justifique o peso na descrição do PR, com o impacto no binário.
- Código e identificadores em inglês; documentação do projeto em português.

**Commits** (proposta, a confirmar)

- Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
- Cada commit ou PR que muda estrutura, comando ou decisão inclui a atualização deste arquivo.

## Riscos conhecidos

| Risco | Impacto | Mitigação |
| --- | --- | --- |
| egui é immediate mode: texto rico, listas longas e IME exigem trabalho extra | Alto | Spike com uma tela de chat real: 10 mil mensagens virtualizadas, emoji, IME e medição de RAM com glow vs wgpu |
| Cliente Rust do Convex recebe argumentos como mapas genéricos, sem tipos gerados | Médio | Camada fina de tipos em Rust; avaliar gerar a partir do `schema.ts` |
| WebRTC em Rust pesa no binário e na complexidade | Alto | Voz isolada em módulo opcional, só na fase 3 |
| Convex self-hosted tem upgrades e backups menos prontos que o cloud | Médio | Fixar versão, documentar backup do SQLite/Postgres |
| Presença e "digitando..." consumindo muitas chamadas de função | Médio | Throttle no cliente; mover para Durable Objects só se medido |

## Decisões

**Já tomadas**

- Cliente em Rust nativo com egui via eframe, sem webview.
- Crate oficial `convex` sobre tokio para dados e tempo real.
- Backend inteiro em funções Convex, com o mesmo código no cloud e em self-host.
- Serviços externos atrás de adaptadores escolhidos por variável de ambiente.
- Autenticação OIDC com JWT validado pelo Convex, sem provedor proprietário obrigatório.
- Renderização com glow: medido bem mais leve que wgpu.
- eframe fixado na 0.36 porque `egui-phosphor` (ícones) ainda não suporta a 0.37.
- Licença: GPL-3.0.
- MVP só de texto; voz na fase 3, em módulo opcional.
- O roadmap tem 5 fases e 4 portões: a fase seguinte só começa quando o critério do portão é atingido. As fases não têm datas.

**Em aberto**

- [ ] Como cumprir a meta de RAM: reduzir o consumo, ou rever a meta considerando o driver de vídeo
- [ ] Provedor de auth da instância oficial e o padrão para self-host
- [ ] SFU recomendado para self-host
- [ ] Detalhe das fases e critérios dos portões do roadmap (ainda não registrados neste repositório)
- [ ] Convenção de commits
- [ ] Acessibilidade: o `accesskit` está desligado no cliente para economizar peso; reavaliar antes da v0.1
