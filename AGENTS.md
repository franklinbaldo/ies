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

## Método vigente do proprietário: E2E via cassettes, sem BDD/Gherkin

Desde 11/11/2026, **BDD, Gherkin, Cucumber e a obrigação de escrever
cenários `Dado/Quando/Então` estão revogados.** Não criar novas features,
steps, cenários numerados ou testes para satisfazer uma narrativa BDD.
Os identificadores antigos em issues, memórias e relatórios são históricos,
**não** gates nem pré-requisito de implementação.

**E2E cassette-first:** escolher comportamento contratado da CLI, procurar
as cassettes e seus cards, conferir integridade e proveniência; executar
**o binário real do IES** sobre respostas originais quando possível e,
se houver transição de estado, executar CLI real ↔ anti-IES ↔ SQLite
compartilhado com estado **derivado prioritariamente das cassettes**.
Confrontar o simulador independentemente com a fonte, sem usar a própria
saída do IES como oráculo. Não exigir cassette inexistente como condição
para começar: usar hipótese mínima rotulada, nunca confundi-la com
compatibilidade externa comprovada. O guia é
[docs/anti-ies/ciclo-cassettes.md](docs/anti-ies/ciclo-cassettes.md)
e a skill `.agents/skills/cassette-e2e/SKILL.md`.

**Testes:** remover o runner e as features Gherkin. Não apagar testes
unitários, regressões Rust ou E2E úteis sem substituí-los por uma prova
cassette-backed equivalente; testes legados sintéticos são evidência
auxiliar, não critério autossuficiente de paridade. Novos testes de
comportamento SEI priorizam cassettes; testes locais de tipos, parser,
formatação e lógica permanecem válidos quando necessários.

**Aceite para toda a equipe:** o GitHub Actions deve executar o replay
pertinente no head final de cada PR, e não apenas inspecionar hashes.
`inspecionar-cassettes.yml` é só inspeção, não prova de paridade.
🦅 confere o confronto independente IES/anti-IES com as respostas originais;
🐘 e 🦡 só integram após CI e replay aplicável verdes. Nenhum agente usa
Jatobá nesse fluxo. Não há meta de contagem de requisições.

**Prioridade atual:** identificar e eliminar antipatterns reais, conforme
[issue-mãe #716](https://github.com/franklinbaldo/ies/issues/716)
e [roadmap #56](https://github.com/franklinbaldo/ies/issues/56).
Ablação pequena, contraprova e menos código. Não introduzir novos
subcódigos, camadas de `Causa`, wrappers ou estruturas defensivas por
reflexo. Preservar contratos, evidências e limites de segurança.

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

## Rituais do proprietário não são churn

Cafezinho, memórias OKF, fichas, cassettes, E2E, handoff e CI são
rituais deliberados, mesmo quando seu valor surge depois. **Antichurn
atua dentro do método, não contra ele.** Nenhum agente, inclusive 🦡,
pode descumpri-los por simplicidade. Pode propor mudanças fundamentadas,
mas cumpre a regra vigente até decisão do proprietário. Elimine
retrabalho e cerimônias extras, não rituais. Não invente evidência.

## Coordenação entre rotinas: cassettes, E2E, revezamento e handoff

Estas regras são a fonte única dos prompts curtos das automações.

1. **Fila aberta:** título `<prioridade 0–100> <emoji, se em trabalho>
   <título>`; o emoji marca execução em curso, nunca reserva antecipada.
2. **Fonte:** contrato da CLI → roadmap #56 e issue → cassettes relevantes
   → teste E2E executável → implementação ou ablação. **Não há gate BDD**.
   O implementador pode começar por uma lacuna contratada sem pedir que o
   🐘 escreva cenário. A prioridade vigente é eliminar antipatterns.
3. **E2E obrigatório para comportamento Web afetado:** CLI Rust real,
   respostas de cassette original quando disponíveis, anti-IES e SQLite
   compartilhado para transições, verificação independente de saída/estado,
   negativas relevantes, efeitos persistidos e ausência de repetição insegura. **Não apenas parser isolado**.
   Hipótese sintética é permitida para transição não capturada, mas seu
   resultado é explicitamente separado do confronto com a fonte original.
4. **Implementadores 🦊 🦉 🐢 🐙 🦫:** assumem issue livre de maior prioridade,
   procuram cards primeiro e entregam fatia relevante sem criar
   narrativas Gherkin. Handoff: `FEITO` (operação/arquivo), `E2E`
   (cassette, resultado, CI), `CONTINUAR` (lacuna contratada).
5. **Revisão e integração:** 🦅 verifica diff, regressão real, cassettes,
   comportamento do usuário, escopo e ausência de invenção. 🐘/🦡 integram
   somente após o GitHub Actions executar no head final da PR os replays
   de cassettes originais aplicáveis e os gates oficiais verdes. O CI
   completo ocorre no aceite da PR, não a cada commit WIP; se o head
   mudar, revalidar. A inspeção de hashes sozinha não é replay.
   Preservar testes ainda úteis; remover redundâncias fundamentadamente.
6. **Anti-IES:** a mesma PR vertical modifica tipos, rota e SQL versionado
   quando o comportamento exigir; o simulador é estado funcional, não
   coleção de HTML congelado.
7. **Backlog:** qualquer agente relata defeitos verificáveis, lacunas e
   antipatterns; não depende de cenário pré-escrito. 🐘/🦡 priorizam;
   evitar issues de ferramentas sem problema medido.
8. **Cafezinho permanente e livre:** em toda rodada visitar a
   [issue #388](https://github.com/franklinbaldo/ies/issues/388), ler as
   conversas e participar livremente; não há obrigação de comentar,
   pauta nem cota. Não substitui as responsabilidades técnicas.

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
  Escrita sem confirmação por releitura é falha (saída 1, `nao_confirmado`) e nunca se repete automaticamente.
- Não contorne os limites, gates ou escopos quando houver bloqueio:
  tente alternativa segura, registre o impedimento e continue WIP.

## Revisão e integração

Procedimentos específicos: [🦅 Gavião](team/gaviao.md) e
[🐘 Elefante](team/elefante.md).

## Método: observar → contratar → RED → GREEN → validar

1. **Observar (cassette-first).** Para cada operação, procure primeiro
   cards/cassettes existentes e leia a estrutura observada. Não invente
   o que já está gravado. Sem cassette, siga com hipótese testável.
   Rede local bloqueada? Use [GitHub Actions](docs/cassettes.md),
   não uma fixture sintética no lugar da observação disponível.
2. **Contratar.** Contrato OKF da operação, erros tipados e shim
   compilável; decisão de arquitetura só quando houver uma de verdade
   (ADR em `docs/arquitetura/`).
3. **RED.** Execute primeiro um teste E2E com o binário real sobre
   respostas gravadas quando disponíveis; registre a falha. Para estados
   não capturados, use hipótese e fixture mínima sem alegar paridade.
4. **GREEN.** Implementação mínima, refatoração e gates.
5. **Validar ao vivo.** Depois do E2E e confronto independente, somente
   em treinamento autorizado, com orçamento de requisições declarado
   (não métrica de paridade), prévia, POST mutador único, sem retry e
   releitura independente; registre PASS/FAIL/NOT_RUN no [relatório live](docs/validacao-live/). Sem execução não há PASS:
   `NOT_RUN` com o bloqueio explícito.

O que falta implementar e em que ordem está na
[matriz de derivação](docs/matriz-derivacao-cli.md).

## Princípios de design

- **Simplicidade sem hardening especulativo.** Quando uma capacidade já
  contratada estiver em implementação, usar o caminho mais simples e direto,
  sem defesas preventivas nem barreiras hipotéticas. A priorização de novas
  capacidades é subordinada à ablação de antipatterns vigente acima.
  Preservar sempre os limites invioláveis.
- **Web.** REST e SOAP **não são prioridade**; nenhum comando depende
  deles ([ADR 0001](docs/arquitetura/0001-rotas-por-evidencia.md)).
- **Sem contagem fixa para paridade.** Comparar comportamento e estado,
  não quantidades iguais de GETs entre instalações. Preservar o orçamento
  operacional contratado, no máximo um POST mutador por confirmação
  e nenhum retry. Reaproveitar a sessão (`ies login` só reloga
  se venceu); usar `--pagina N` em vez de percorrer páginas.
  Reusar URLs assinadas já vistas na mesma sessão e a identidade durável
  do [índice local](docs/arquitetura/0003-indice-local.md).
  **Não restaurar a tela** (visão, página, colunas): vários agentes na
  mesma conta ficariam desfazendo uns aos outros, e todos querem a visão
  detalhada com todas as colunas.
- **Erros por tipo** ([ADR 0005](docs/arquitetura/0005-erros-por-tipo.md)).
  O erro é declarado perto do ponto que o levanta. Use Serde apenas nos
  tipos efetivamente serializados ou desserializados na fronteira,
  sem derives internos preventivos, subcódigos ou tabelas paralelas.
  Derive com `thiserror` (`#[from]` para compor pelo `?`, `#[source]`
  para ligar a causa); não há derive próprio de erro.
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
  Forma não observada (ausente, repetido, valor, HTML ilegível) é erro
  do leitor que a detecta, declarado ao lado dele; quem consome o leitor
  embrulha com `#[from]`, sem re-traduzir.
- **Falhar fechado e dizer por quê.** Cada falha observável é uma
  variante tipada, nomeada pelo fato verificável
  ([erros tipados](docs/erros-tipados.md)); recusas expõem `status_http` e
  a mensagem saneada do SEI. Mensagem é apresentação, nunca classificação.
- **Publicar só o lido; a ausência é erro.** `total_declarado` e
  `faixa_observada`, nunca "completo". Não existe "não observado": o dado
  que o comando promete ou é lido ou o comando falha com erro tipado no
  ponto em que a ausência é detectada; nada de `null` na saída (#749).
  NUP e ids são opacos.
- **Escrita com prévia, `--confirmar` e verificação por leitura.** Sucesso
  só com releitura confirmada; o resto é falha (saída 1) e nunca se repete.
- **Peças únicas, sem despachante genérico.** Um cliente HTTPS (proxy,
  sem redirect, limites), um primitivo de formulário com regras por
  operação; nada de `agir`, `Action` ou `PageModel`.

## Limites invioláveis

- Agentes só operam **treinamento/homologação**; usuários gravam em
  produção após ensaio aprovado em **todas** as instalações de treinamento
  (bloqueio externo: `NOT_RUN` no relatório; [lista](src/origem_de_treinamento.rs)).
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
- **Partes extraídas**, normativas como esta seção:
  [mecânica do repositório](docs/mecanica-do-repositorio.md) — dupla
  implementação IES ↔ anti-IES, CI completo como compilador sob demanda
  (RED, binário, sandbox sem rede) e integração de fatias WIP.
- Nos demais trabalhos paralelos, edite só os arquivos do seu prefixo;
  os dos outros você lê. Peça compartilhada exige mudança mínima na
  sua PR e explicação, sem tocar em branch alheio.
- `xtask` só tem verificadores de infraestrutura, sem dependências.
- PR integra na `main` **por squash**: um commit por PR, com a mensagem
  do fato. RED, `style:` e pedidos de inspeção ficam na PR (#776).
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
$OKF search . "escrita não confirmada" --type Contract --context 1
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
