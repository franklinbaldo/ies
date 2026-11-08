---
type: Guide
title: "anti-IES — engenharia reversa do SEI com REA"
description: "Evidências de interface Web, modelo funcional compartilhado e transições comprovadas"
---

# Engenharia reversa comportamental: REA → modelo funcional

O anti-IES deve comportar-se como **um sistema simulado de SEI**, não como
uma biblioteca de mocks, endpoints independentes ou páginas HTML armazenadas.
O IES alimenta SQLite efêmero com cassettes ou fixtures sintéticas;
o anti-IES executa **transições testáveis mesmo antes de capturas**.
Hipóteses são corrigidas conforme testes e evidências posteriores.
Um processo, uma unidade e um retorno referem-se às mesmas entidades entre
operações quando a identidade comum foi efetivamente observada.

## Ferramenta REA: usar onde acrescenta evidência

Skill oficial: [Reverse Engineer Anything](https://github.com/morluto/rea/tree/main/.agents/skills/reverse-engineer-anything).
REA analisa **artefatos entregues, capturas e runtime**, não substitui uma
leitura normal do repositório Rust, conforme a própria skill. A execução de
`inspect_web_network_capture` aceita **HAR 1.2**, `saz` e outros formatos
que a ferramenta efetivamente anunciar; **a cassette JSON do acervo IES
não é HAR**. Não passar JSON histórico como HAR, nem inventar campos
obrigatórios para converter uma fonte parcial em evidência completa.

Na máquina de desenvolvimento **autorizada** que usa Codex:

```sh
npx -y rea-agents@latest setup --client codex --dry-run --json
# Revisar o plano e aprovar a instalação/conexão em sessão local:
npx -y rea-agents@latest setup --client codex
# Alternativa SOMENTE para instalar as instruções da skill, sem MCP:
npx skills add morluto/rea --skill reverse-engineer-anything
```

A instalação requer Node.js compatível conforme a versão publicada e
reconexão/reinício do cliente após alterar MCP. **Não** automatizar setup
interativo no CI, instalar disassemblers sem necessidade ou supor que a skill
sozinha habilita um servidor REA. Se o MCP não estiver conectado, trabalhar
com os verificadores de cassettes já disponíveis no IES, documentar
`REA: NOT_RUN` e não chamar o relatório de evidência produzida por REA.

Para HAR previamente capturado em **treinamento autorizado**, saneado antes
da análise (dados de sessão fora da cópia), a operação correspondente é:

```sh
rea inspect-web-network-capture /caminho/absoluto/ensaio.har har --json
```

O registro REA deve reter identidade/digest do artefato, ordinais,
cobertura, limitações, exclusões de credenciais e desconhecidos.
Uma captura histórica não executa requisições. Em ambiente descartável
autorizado, `capture_browser_scenario` pode estudar interação, mas
escrita só com autorização, orçamento e releitura do efeito. Um
`AgentEnv` produzido por REA é **oráculo auxiliar**, não uma cópia
automática do backend SEI nem substituto das entidades Rust.

## O que reconstruir

Para cada capacidade contratada da CLI, a investigação precisa distinguir:

- **Entidades:** instalação, usuário/unidade, processo, documento, andamento,
  retorno, situação, vínculos e identidades observadas. ID/NUP opaco,
  `Option` para campos ausentes e `enum` para alternativas fechadas.
- **Estado:** relacionamentos, existência, versões, permissões observadas,
  curso do processo e visões derivadas, com histórico de proveniência.
- **Operações e transições:** requisição observada, pré-condições,
  resposta, efeito confirmado pela releitura e efeitos secundários.
  GET potencialmente mutante também requer observação de efeito.
- **Incerteza:** resposta HTTP 200, HTML após POST ou corpo de requisição
  ausente **não provam** gravação, causalidade, completude ou igualdade de
  estado. Não implementar escrita presumida para satisfazer teste.
- **Representação:** HTML/JSON são projeções desse estado. Não duplicar um
  snapshot por endpoint nem copiar bytes da resposta capturada.

O esquema permanece em `crates/iesei/migrations/NNNN_<capacidade>.sql`.
Novas entidades e relações **podem nascer como hipóteses testadas**,
sem esperar cassette; tipos, SQL, ingestão e servidor evoluem na mesma
PR vertical, sendo revistos diante de novas evidências.
Dados do SQLite de ensaio são descartáveis; a cassette verificada é a
fonte durável. Não misturar com `ies.sqlite3` operacional nem expor
`cookie`, `infra_hash`, URLs assinadas ou HTML autenticado.

## Prova de que virou um sistema

1. **Extração:** resposta gravada e hashes verificados → fatos tipados
   pelo leitor IES, com proveniência e desconhecidos preservados.
2. **Coerência de leituras:** sem trocar banco ou escolher manualmente
   uma fixture por chamada, duas operações consultam o **mesmo estado**
   de instalação/cenário. Não afirmar vínculo de identidade sem fonte.
3. **Transição funcional:** implementar a mudança de estado, testar
   leitura posterior e recusas. Manter a hipótese revisável até ser
   confrontada com o SEI gravado.
4. **Comparação independente:** resultado do IES nas respostas originais
   versus o IES contra HTTP do anti-IES, preservando códigos, causas,
   contagens e relações. Um parser isolado é **COMPONENTE**, nunca E2E
   da CLI com TLS/sessão.
5. **Ablação:** substituir mocks redundantes **apenas após** provar
   cenários positivos/negativos equivalentes. Sem nova suíte global
   paralela, sem baixar ZIP em toda PR, sem abrandar CI.

Cada PR informa hipótese, testes e cobertura reproduzível, versão
do shape, SHA do CI e efeitos. Fonte/digest e `REA: PASS/FAIL/NOT_RUN`
quando aplicáveis. **Testes são evidência de funcionamento sintético**.
Falta de observação externa não bloqueia a execução; não declarar
hipótese como compatibilidade SEI já demonstrada.
