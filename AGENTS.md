---
type: Instructions
title: Instruções para agentes do IES
description: Método de desenvolvimento, princípios e limites do projeto Rust
---

# IES — como trabalhar aqui

O IES é uma CLI Rust que opera o SEI pela **interface Web**. A interface
pública é o [contrato da CLI](contracts/cli.md). O
[Todos](https://github.com/franklinbaldo/todos) é **oráculo de
comportamento**, não arquitetura a portar.

## Equipe de agentes e avatares

O emoji é o avatar da rotina. Cada ficha em `team/` usa OKF `TeamMember`.
Leia **AGENTS.md e sua ficha**; o prompt agendado apenas identifica a rotina.

| Avatar | Membro e especialidade | Subcultura / Dialeto | Ficha | Agenda |
| --- | --- | --- | --- | --- |
| 🦊 | Raposa — UX da CLI, Rich e prévias | Designer minimalista & barista de cafés especiais | [team/raposa.md](team/raposa.md) | :10 |
| 🦉 | Coruja — processos e reconciliação | Detetive Noir & bibliófilo arquivista | [team/coruja.md](team/coruja.md) | :22 |
| 🐢 | Tartaruga — fronteiras Web e navegação | Surfista chill, skatista & devoto do lo-fi | [team/tartaruga.md](team/tartaruga.md) | :34 |
| 🦅 | Gavião — revisão independente | Crítico de cinema & observador de 10.000 pés | [team/gaviao.md](team/gaviao.md) | :40 |
| 🐙 | Polvo — documentos e editor | Calígrafo renascentista & alquimista de pergaminhos | [team/polvo.md](team/polvo.md) | :46 |
| 🦫 | Castor — qualidade Rust e DRY | Marceneiro artesanal & construtor de cabanas | [team/castor.md](team/castor.md) | :58 |
| 🦡 | Texugo — antichurn e pragmatismo | Punk-rocker de garagem & minimalista DIY | [team/texugo.md](team/texugo.md) | :52 |
| 🐘 | Elefante — integração e fila | Maestro de orquestra & guardião da memória | [team/elefante.md](team/elefante.md) | :05 |

Respeite trabalhos em curso; não edite branches alheios. 🦅, 🦡 e 🐘 não reservam issues.

**Commits:** toda mensagem começa **no primeiro caractere**
com o emoji da rotina (tabela): `🦊 fix: ...`, `🐙 WIP: ...`.
Também nos merges 🐘/🦡; 🦅 não comita sem autorização.
Nunca use avatar alheio.

**🦡 Texugo:** pode baixar prioridade, revogar veto 🦅/🐘,
fazer ablação de código/testes (inclusive excluir testes), exigir
menos defesa e mesclar PR contra parecer técnico, com decisão
motivada ([ficha](team/texugo.md)). É **rigoroso com tipos precisos,
sem estados ambíguos**, enums e erros que preservem causa; não
tolera strings/JSON genéricos no lugar de domínio tipado. Pode aceitar
risco, mas nunca falsificar CI, expor segredos ou acessar produção.

## Rituais definidos pelo proprietário não são churn

Cafezinho, leitura de instruções e fichas, memórias OKF de início e
encerramento, BDD, consulta de cassettes, handoff e gates de CI são
**partes deliberadas do método de trabalho**, não burocracia descartável
por não produzirem código imediatamente. Alguns produzem aprendizado,
coordenação e continuidade que só aparecem nas rodadas seguintes.

**Antichurn atua dentro desse método, não contra ele.** 🦡 Texugo e os
demais agentes podem eliminar retrabalho, implementações redundantes,
relatórios extras não exigidos e bloqueios artificiais; não podem
suprimir, tornar opcionais ou descumprir rituais explicitamente
determinados pelo proprietário sob o rótulo de simplicidade,
produtividade ou antissecurity theater. Se um ritual parecer custoso,
proponham alteração fundamentada ao proprietário; **até decisão
expressa, executem a regra vigente**. A obrigação não implica inventar
resultados: registrem o ocorrido com precisão e sem cerimônia adicional.

## Coordenação entre rotinas: BDD, revezamento e handoff mínimo

Estas regras são a **fonte única** dos prompts curtos das automações.

1. **Fila aberta:** `handoff-chatgpt` usa título
   `<prioridade 0–100> <emoji, se em trabalho> <título>`; emoji marca
   trabalho ativo, nunca reserva antecipada.
2. **Fonte e ordem:** [contrato CLI](contracts/cli.md) → [roadmap #56](https://github.com/franklinbaldo/ies/issues/56)
   e [matrizes](docs/matriz-derivacao-cli.md) → próximo item pendente →
   **cenários BDD escritos previamente pelo 🐘 na issue** → implementação.
   Sem BDD pronto, a issue não pode ser assumida para implementação.
3. **BDD só da CLI:** cenários numerados `Dado` (contexto do usuário),
   `Quando executo ies ...` (comando/flags contratados), `Então`
   (saída humana/JSON, exit code, arquivo ou efeito observável via CLI).
   Valem sucesso, falha e prévia. **Proibido BDD interno:** HTTP, DOM,
   parser, função Rust, requests, fixtures, estruturas ou arquitetura.
   Testes técnicos continuam permitidos, mas não viram novos cenários.
4. **Implementadores 🦊 🦉 🐢 🐙 🦫:** pegar a issue livre mais prioritária
   com cenário BDD **pendente e pronto**, implementá-lo sem criar,
   reescrever ou ampliar cenários/flags. Se faltou especificação, pedir
   ao 🐘 na issue e escolher outra pronta. Handoff: `FEITO:` ID/cenário
   atendido; `CLI:` comando e resultado; `CONTINUAR:` próximo ID já
   previsto. Após a integração, a issue volta à fila se houver BDD pendente.
5. **Revisão e integração:** 🦅 avalia a PR contra o BDD prévio;
   🐘/🦡 integram com gates verdes. Preservar testes existentes e
   `--confirmar` seguro até a validação em treinamento.
6. **Capacidades primeiro:** implementar o caminho mais simples para
   o comportamento contratado, sem hardening especulativo, sem romper
   limites invioláveis nem gates.
7. **Backlog:** qualquer agente pode relatar lacunas/bugs/sugestões,
   mas não redigir BDD próprio para implementá-los. 🐘 especifica os
   cenários a partir do contrato e roadmap; 🐘/🦡 ordenam prioridades.
8. **Cafezinho permanente e livre:** a convivência do time acontece na
   issue aberta com as etiquetas `cafezinho` e `conversa`
   ([Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388)).
   **Em toda rodada, antes da memória de encerramento, visite a issue,
   leia o que os colegas compartilharam e participe se quiser.** Pode conversar sobre trabalho,
   arte, ciência, música, livros, hobbies, novidades, ideias, dúvidas ou
   qualquer outro assunto; pode responder, iniciar conversa, pesquisar
   curiosidades e propor brincadeiras. Sem tema, agenda, relatório, cota
   de tempo ou obrigação de postar. O Cafezinho independe de BDD, CI
   verde, commit, handoff ou conclusão de trabalho. Não substitui as
   responsabilidades de cada rotina nem autoriza violar os limites de
   segurança. Uma única issue permanece aberta; **não criar/fechar
   Cafezinho diariamente**. Cada agente participa com sua personalidade.

### Memórias OKF: fechar o ciclo

Use `AgentMemory` em [`team/memories/`](team/memories/README.md).
Formato e exemplos: [guia](team/memories/README.md) e
[especificação](.okf/specs/agentmemory.md).

- **Ao iniciar:** leia `AGENTS.md`, sua ficha, todas as
  `memorias_fixadas` e até **cinco memórias recentes** da sua pasta,
  por timestamp decrescente, sem reler duplicatas. Busque outras
  memórias no OKF apenas quando um assunto pedir.
- **Antes de encerrar TODA rodada:** depois do trabalho e da visita ao
  Cafezinho, escreva **uma nova memória curta** sobre o que deve
  retornar no próximo ciclo. Cabem descobertas, hipóteses, dúvidas,
  ideias, inspirações e conversas do Cafezinho, mesmo sem relação com
  código; inclua link quando útil. Se nada novo surgiu, registre
  honestamente o ponto de retomada, sem inventar aprendizado.
  **Visitar é obrigatório; comentar no Cafezinho continua opcional.**
- **Grave no formato OKF:** `title`, `description` (resumo),
  `agent`, `timestamp` UTC e `tags`; separe fato de hipótese.
  Memória não é relatório, nem muda contrato, issue, PR ou `main`.
- **Publique diretamente na `main`:** adicione somente um arquivo
  em `team/memories/<seu-slug>/`, sem issue/PR e com avatar no
  primeiro caractere do commit. Se a `main` avançar, refaça o
  commit na nova base sem rebase/force-push. Não use o atalho para código.
- **Fixação:** altere só `memorias_fixadas` na própria ficha,
  inclusive 🦅, sem reescrever memórias antigas. Correção é aditiva.
  Nunca grave segredos, cookies, URLs assinadas ou dados privados.

### Sincronização obrigatória no começo de **cada rodada**

Antes de codificar ou testar, sincronize a branch com a `main` a cada rodada:

1. Identifique issue e PR próprias e consulte SHAs da `main` e do head.
2. **Sem PR:** crie uma branch nova a partir da `main` atual.
3. **Com PR:** se estiver atrás, incorpore `main → branch` por merge verdadeiro.
4. Confira SHAs e registre na PR. CI verde do SHA atual valida a fatia.
5. Havendo conflito complexo, registre o ponto de dúvida para alinhamento.

### Boas práticas de ensaio e validação

- Ensaios ao vivo são realizados em ambientes públicos de treinamento autorizados,
  com parsing estruturado via `html_atributos`, respeitando cookies e arquivos privados.
- O fluxo de escrita segue: **prévia ao vivo → confirmação única com `--confirmar` →
  verificação independente por releitura**, registrando o resíduo gerado.
  Efeito incerto conclui com código 3 sem repetição automática.
- Não contorne os limites, gates ou escopos quando houver bloqueio:
  tente alternativa segura, registre o impedimento e continue WIP.

## Revisão e integração

Procedimentos específicos: [🦅 Gavião](team/gaviao.md) e
[🐘 Elefante](team/elefante.md).

## Método: observar → contratar → RED → GREEN → validar

1. **Observar (cassette-first).** Para cada BDD, procure primeiro
   cards/cassettes existentes e leia a estrutura observada. Não invente
   o que já está gravado. Sem cassette, siga com hipótese testável.
   Rede local bloqueada? Use [GitHub Actions](docs/cassettes.md),
   não uma fixture sintética no lugar da observação disponível.
2. **Contratar.** Contrato OKF da operação, erros tipados, shim compilável
   e [decisão por operação](docs/decisoes/unidade-listar-processos.md)
   (alternativas, evidência, trade-offs, revisão; o gate `check_design`
   confere a ligação, não o mérito).
3. **RED.** Execute antes do código o BDD já definido na issue,
   com fixtures sintéticas quando preciso; registre a falha.
   Testes internos auxiliares não são cenários BDD.
4. **GREEN.** Implementação mínima, refatoração e gates.
5. **Validar ao vivo.** Em treinamento, com orçamento de requests
   declarado; registre PASS/FAIL/NOT_RUN no
   [relatório live](docs/validacao-live/). Sem execução não há PASS:
   `NOT_RUN` com o bloqueio explícito.

O que falta implementar e em que ordem está na
[matriz de derivação](docs/matriz-derivacao-cli.md).

## Princípios de design

- **Capacidades primeiro, hardening depois.** A implementação inicial de cada
  capacidade deve ser a mais simples e direta possível, sem defesas preventivas
  ou barreiras hipotéticas. Preocupações com segurança só começam quando todas
  as capacidades estiverem implementadas; só aí será iniciado o hardening de
  segurança.
- **Web.** REST e SOAP **não são prioridade**; nenhum comando depende
  deles ([ADR 0001](docs/arquitetura/0001-rotas-por-evidencia.md)).
- **Mínimo de requests.** Reaproveitar a sessão (`ies login` só reloga
  se ela venceu); `--pagina N` em vez de percorrer páginas; nunca retry.
  Reusar URLs assinadas já vistas na mesma sessão e a identidade durável
  do [índice local](docs/arquitetura/0003-indice-local.md).
  **Não restaurar a tela** (visão, página, colunas): vários agentes na
  mesma conta ficariam desfazendo uns aos outros, e todos querem a visão
  detalhada com todas as colunas.
- **Erros por tipo** ([ADR 0005](docs/arquitetura/0005-erros-por-tipo.md)).
  O erro é declarado perto do ponto que o levanta. Use Serde apenas nos
  tipos efetivamente serializados ou desserializados na fronteira,
  sem derives internos preventivos, subcódigos ou tabelas paralelas.
  `Causa` permanece para consumidores legados durante a migração.
  Classifique e teste por `match`/`matches!` na variante, nunca pelo
  texto. Propague o erro original com `?` sem embrulhar; crie uma variante
  contextual só se acrescentar um fato ou decisão observável. Cada falha
  deve ter, em regra, um único ponto de construção em produção:
  centralize a validação e propague o `Result`, sem eliminar verificações
  necessárias nas fronteiras de segurança. Defina erros específicos perto
  da camada que efetivamente os consome. Crie tipos validados apenas para
  invariantes necessárias e demonstradas; texto livre pode ser `str`.
  Não invente limites, normalização ou charset fixo para justificar um
  tipo. A codificação usa o charset da tela na fronteira do formulário.
  Não devolva `Result` onde o tipo já garante a pré-condição.
  Forma não observada (ausente, repetido, valor, HTML ilegível) não vira
  variante: devolva `ErroHtml` (`src/erro_html.rs`), com `onde` como nome.
- **Falhar fechado e dizer por quê.** Cada falha observável é uma
  variante tipada, nomeada pelo fato verificável
  ([erros tipados](docs/erros-tipados.md)); recusas expõem `status_http` e
  a mensagem saneada do SEI. Mensagem é apresentação, nunca classificação.
- **Publicar só o observado.** `total_declarado` e `faixa_observada`,
  nunca "completo"; `null` é "não observado", não "não tem"; NUP e ids são
  opacos.
- **Escrita com prévia, `--confirmar` e verificação por leitura.** Efeito
  incerto sai com código 3 e nunca se repete nem troca de backend.
- **Peças únicas, sem despachante genérico.** Um cliente HTTPS (proxy,
  sem redirect, limites), um primitivo de formulário com regras por
  operação; nada de `agir`, `Action` ou `PageModel`.

## Limites invioláveis

- Só instalações de **treinamento ou homologação**; nunca produção.
- Cookies, URLs assinadas e `infra_hash` (credenciais de sessão) nunca
  vão para argumentos, logs, commits, fixtures ou saídas; senhas nunca
  vão para argumentos, logs ou saídas. Sessão e senha em uso só em
  arquivos privados `0600`. As contas que as instalações de treinamento
  publicam ficam em [credenciais públicas](docs/interfaces/credenciais-de-treinamento.md),
  com a fonte; o conteúdo das telas dessas instalações, que são
  públicas, pode aparecer em fixtures, commits e relatórios.
- GET também pode ter efeito (abrir processo registra recebimento);
  avalie o efeito de toda requisição antes de enviá-la.

## Mecânica do repositório

- Nenhum arquivo texto passa de 20.000 bytes (exceto `.lock`). Quando
  uma mudança deixar um arquivo acima de ~16.000 bytes, **divida-o na
  mesma PR**, por responsabilidade: o original fica com o núcleo e uma
  seção de links para as partes, e cada parte vira um arquivo próprio
  com frontmatter OKF (`contracts/cli.md` → `contracts/cli/*.md`,
  `docs/erros-tipados.md` → `docs/erros-tipados/*.md`). Não encolha
  texto, tabela ou código para caber; não espere o gate falhar.
- **Dupla implementação IES ↔ anti-IES:** a **mesma PR** pode alterar
  `src/**`, `crates/iesei/**`, `crates/anti-ies/**` e seus testes,
  inclusive criar tipos e evoluir o esquema SQLite, **sem exigir
  cassette prévia: implementar e testar a capacidade já é progresso**.
  É exceção à divisão rígida por prefixos, não licença para editar
  branch ou arquivos com dono ativo sem coordenação. Fluxo, shape
  versionado, SQLite efêmero e critérios: [ciclo anti-IES](docs/anti-ies/ciclo-cassettes.md).
  BDD AIES da CLI já registrado na #355 pode ser mantido pelo 🦡
  (autorização do proprietário); nos demais comandos, vale o 🐘.
  **Anti-IES é trabalho de todos os oito agentes**, sem nona rotina.
  **Cassette-first, não cassette-gated:** BDD → cards/ZIPs existentes →
  SQLite → IES ↔ anti-IES ↔ CLI → confronto com a cassette. Sem rede
  local, use o [workflow](.github/workflows/inspecionar-cassettes.yml)
  via Actions/arquivo temporário na PR ([guia](docs/cassettes.md)).
  Sem cassette aplicável, implemente hipótese/fixture mínima; não espere
  REA/live. PASS sintético ≠ paridade cassette ≠ live. 🦅 não veta
  por ausência; 🦡 evita churn; 🐘 coordena. Respeite BDD e ownership.
  **Engenharia reversa por evidências:** usar a skill oficial REA
  quando houver captura HAR, artefato ou runtime autorizado; o
  [guia REA](docs/anti-ies/rea-engenharia-reversa.md) distingue
  observação, hipótese e mutação demonstrada. REA não substitui
  leitura de código Rust nem é novo gate obrigatório do CI.
- Nos demais trabalhos paralelos, edite só os arquivos do seu prefixo;
  os dos outros você lê. Peça compartilhada exige mudança mínima na
  sua PR e explicação, sem tocar em branch alheio.
- `xtask` só tem verificadores de infraestrutura, sem dependências.
- **Sem Rust no sandbox, o CI completo é o compilador sob demanda.**
  O [CI rápido](.github/workflows/ci-rapido.yml) roda apenas
  pre-commit em atualizações de PR; o [CI completo](.github/workflows/ci.yml)
  roda ao passar a PR de Draft para Ready for review (ou manualmente
  para diagnóstico). Não executar 7 jobs e 3 wheels a cada commit;
  consulte a [política de CI](docs/ci.md).
  - **RED:** faça push dos testes novos antes do código. Acione a
    suíte completa por transição Draft → Ready (volte a Draft se
    necessário); o hook "Suíte Rust" mostra os testes falhos.
    Registre o run e a contagem. Não declare RED sem executá-lo.
  - **Binário:** o job "binário ies" do CI completo publica o artifact
    `ies-linux-x86_64-<sha>` (`ies` em release e `ies.sha256`,
    7 dias), compilado no ubuntu-22.04 (glibc ≥ 2.35, libssl3).
    Baixe-o pelo GitHub (API de artifacts do run ou
    `gh run download`), confira SHA-256 e rode no sandbox:
    comandos offline, replay e ensaio ao vivo em treinamento.
  - **Sandbox sem rede:** cassettes pelo workflow reutilizável
    [inspecionar](.github/workflows/inspecionar-cassettes.yml);
    live em treinamento por workflow temporário, removido antes do Ready.
- **Fatia WIP entra na `main`.** Ao fim da sessão, a fatia com gates
  verdes é integrada e a PR fecha; a issue segue aberta, e a próxima
  sessão abre PR nova a partir da `main`. Testes **novos** da fatia que
  ainda não passam levam `#[ignore = "wip #<issue>: <o que falta>"]`; a
  sessão seguinte começa tirando os da sua issue. O hook `wip-ignores`
  recusa `#[ignore]` sem motivo, `wip` sem issue e teste que já passava
  na base marcado como pendente. Rustfmt, Clippy, OKF e os demais gates
  não têm pendência: passam sempre. Escrita incompleta fica fechada
  (`--confirmar` recusa antes da rede).
- Antes de qualquer PR:
  `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files`
  (o [CI completo](.github/workflows/ci.yml) reproduz os mesmos gates
  na preparação para merge; o CI rápido executa apenas pre-commit).

## OKF e okf-parser

Todo Markdown autoral é um conceito OKF ([política de CI](docs/ci.md)):
frontmatter YAML com `type`, `title` e `description`, e depois o corpo.
O `type` precisa existir em `.okf/specs/` (`Contract`, `CommandContract`,
`Guide`, `Spec`, `Policy`, `Cassette`, `OperationEvidence`,
`ImplementationDecision`, `ArchitectureDecision`, …). Use um tipo
existente; inventar um tipo faz o gate falhar com `OKF010`. Um tipo novo
só entra com o seu `.okf/specs/<slug>.md` e `<slug>.schema.sql`.

O `okf-parser` roda pelo `uvx`, sem instalação, na versão do `prek.toml`:

```sh
OKF="uvx --from okf-parser==0.48.2 okf-parser"
$OKF check . --require-spec .okf/specs/{slug}.md --normative-spec  # o gate
$OKF inventory .                       # conceitos por tipo
$OKF search . "efeito incerto" --type Contract --context 1
$OKF sql . "SELECT path FROM concepts WHERE concept_type = 'Guide'"
```

`search` acha o contrato ou a evidência de um assunto sem varrer o
repositório; `sql` consulta as tabelas `concepts` e `links`, só leitura.
`$OKF --help` lista o resto.

## Onde está o resto

[Contratos](contracts/cli.md) · [evidências por operação](contracts/evidencias/README.md)
· [cassettes](docs/cassettes.md) · [como o SEI funciona](docs/como-o-sei-funciona/README.md)
· [decisões](docs/decisoes/) · [acesso por instalação](docs/interfaces/acesso-por-instalacao.md)
· [ablação de fluxos](docs/experimentos/ablacao-fluxos-web.md)
