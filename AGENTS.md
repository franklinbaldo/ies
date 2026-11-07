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

O workspace também abriga o **anti-IES**, servidor Rust de compatibilidade
incremental ([plano conjunto](docs/anti-ies/roadmap.md)). Os contratos
observados vêm do SEI de treinamento/cassettes, não de concordância entre
cliente e simulador.

## Equipe de agentes e avatares

O emoji é o avatar da rotina. Cada ficha em `team/` usa OKF `TeamMember`.
Leia **AGENTS.md e sua ficha**; o prompt agendado apenas identifica a rotina.

| Avatar | Membro e especialidade | Ficha | Agenda |
| --- | --- | --- | --- |
| 🦊 | Raposa — UX da CLI, Rich, mensagens e prévias | [team/raposa.md](team/raposa.md) | :10 |
| 🦉 | Coruja — processos e reconciliação | [team/coruja.md](team/coruja.md) | :22 |
| 🐢 | Tartaruga — segurança, fronteiras Web e privacidade | [team/tartaruga.md](team/tartaruga.md) | :34 |
| 🦅 | Gavião — revisão independente | [team/gaviao.md](team/gaviao.md) | :40 |
| 🐙 | Polvo — documentos e editor | [team/polvo.md](team/polvo.md) | :46 |
| 🦫 | Castor — qualidade Rust, DRY e tipagem de erros | [team/castor.md](team/castor.md) | :58 |
| 🦡 | Texugo — antichurn e pragmatismo | [team/texugo.md](team/texugo.md) | :52 |
| 🐘 | Elefante — integração, prioridades e fila | [team/elefante.md](team/elefante.md) | :05 |

Preserve reservas; não edite branches alheios. 🦅, 🦡 e 🐘 não reservam issues.

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

## Coordenação entre rotinas: reserva, prioridade e continuidade

Estas regras são a **fonte única** dos prompts curtos das automações.

1. A fila usa etiqueta `handoff-chatgpt` e título
   `<prioridade 0–100> <emoji de dono, se houver> <título>`.
   Maior valor = maior prioridade; 🐘 administra, 🦡 pode reduzir com
   justificativa sem contrariar o proprietário. `em-andamento` não reserva.
2. Cada implementador 🦊🦉🐢🐙🦫 tem **no máximo uma issue** aberta.
   Procure primeiro issue com **seu** emoji imediatamente após o número,
   mesmo com prioridade reduzida. Com duas, continue apenas a de menor
   número e sinalize duplicidade; não assuma outra antes de concluir.
3. Leia issue, comentários (especialmente `Como trabalhar`), histórico da
   PR, `AGENTS.md` e ficha. Retome **o mesmo branch e a mesma PR** se
   ainda abertos. Quando o 🐘 registrar `fatia integrada na main`,
   continue **a mesma issue** em **PR nova a partir da main**, começando
   por testes `wip #N`.
4. Só sem issue própria, escolha issue livre (sem emoji de dono) da
   especialidade, de maior prioridade e, em empate, de **menor número**.
   Se não houver issue livre especializada, escolha a maior prioridade
   geral; nunca a mais antiga como desempate. Examine PRs relacionadas.
5. Reserve inserindo **só o seu emoji** após o número, sem mudar a
   prioridade nem o resto do título. **Releia imediatamente** para
   detectar disputa; se outra rotina venceu, retire apenas sua inserção
   quando isso for seguro, sem apagar o emoji alheio, e escolha outra.
   Desistência insuperável: remova só seu emoji e registre tentativas e
   bloqueio na issue. Não substitua dono porque a prioridade mudou.
6. Em cada execução, entregue **fatia significativa**, RED antes da
   implementação, commits e push, e comentário na issue contendo:
   realizado, restante, CI/SHA, RED e contagens, live
   PASS/FAIL/NOT_RUN, requisições, resíduos, travas e próximos passos.
   Mantenha WIP e reserva até completar os critérios integrais; não
   alegue PASS de execução não realizada.
7. **Implementadores não fazem merge ou fechamento de issues.**
   Não desligue testes para esconder regressões; **🦡 pode remover testes
   em ablação explícita justificada**, com CI válido. Teste novo pendente pode ter
   `#[ignore = "wip #N: motivo específico"]` com o gate correspondente.
   Escrita incompleta deve manter `--confirmar` fechado **antes de
   abrir sessão ou rede**.

### Sincronização obrigatória no começo de **cada rodada**

Antes de modificar código, executar testes RED ou iniciar ensaios, **toda
rotina implementadora** deve sincronizar **sua branch de trabalho** com
a `main` mais recente. Faça isso a cada execução, mesmo que a branch tenha
sido atualizada na sessão anterior:

1. Leia `AGENTS.md` e sua ficha, identifique a issue e a PR próprias e
   consulte os SHAs atuais de `main` e do head da PR.
2. **Sem PR/branch ainda:** crie uma branch nova **a partir da `main`
   atual**, não de uma base salva em sessão anterior.
3. **Com PR aberta:** compare o head com `main`; se estiver atrás,
   incorpore **`main` → sua branch** pelo botão/API de atualizar a PR
   (merge) ou por `git fetch origin && git merge --no-ff origin/main`.
   Resolva conflitos preservando o trabalho já integrado e o de sua
   issue. **Nunca use rebase, reset destrutivo ou force-push.**
4. Confira novamente os SHAs e registre o merge na PR/issue. Rode os
   gates e testes no novo head; CI verde **do SHA antigo** não valida
   o SHA pós-merge. Se o CI falhar, trate a causa antes de solicitar
   integração. A revisão do 🦅 também deve corresponder ao head atual.
5. Se não houver acesso à atualização segura, ou se um conflito não
   puder ser resolvido com evidência suficiente, **não sobrescreva
   alterações de outra rotina**. Registre o impedimento e faça apenas
   investigação independente até conseguir sincronizar.

Vale também para WIP. 🦅 verifica sincronização; 🐘 segue seu protocolo.

### Segurança do ensaio e do contrato

- **Sem produção, sem retry contra SEI, sem execução de JavaScript,
  sem regex para HTML.** Atributos HTML passam por `html_atributos`,
  nunca por extração paralela. Respeite origem de sessão, redirects,
  `arquivo_privado`, `diretorio_do_ies` e `diretorio_do_usuario`.
- Escrita autorizada **somente em TREINAMENTO**, nesta ordem:
  **prévia ao vivo → exatamente um `--confirmar` → verificação
  independente por leitura**. Registre na evidência live o resíduo SEI
  criado/alterado. Sem segurança para completar, falhe fechado sem
  escrever; efeito incerto sai 3, jamais seguido de retry.
- A ciência pertence ao **usuário da sessão**; não exija confirmação
  manual desse fato. HTTP 200 ou item concorrente não é prova suficiente
  de escrita. Relatório live inclui o número **real** de requisições
  observadas; orçamento sintético não substitui observação.
- Não contorne os limites, gates ou escopos quando houver bloqueio:
  tente alternativa segura, registre o impedimento e continue WIP.

## Protocolo de revisão — 🦅 Gavião

**Fonte normativa integral:** [protocolos de revisão e integração](docs/politicas/protocolos-revisao-integracao.md).
🦅 deve ler essa política antes de revisar cada head, além de AGENTS.md e sua ficha.

## Protocolo de integração — 🐘 Elefante

**Fonte normativa integral:** [protocolos de revisão e integração](docs/politicas/protocolos-revisao-integracao.md).
🐘 deve ler essa política antes de integrar, atualizar a fila ou fechar issues.

## Ciclo conjunto do IES e anti-IES

O `iesei` reúne tipos e SQLite; o IES grava a
[projeção](docs/anti-ies/sqlite-companion.md), o anti-IES a lê;
não consulta o SEI real nem altera o contrato da CLI. Para cada capacidade
Web trabalhada no IES, anote na issue/PR a cobertura anti-IES:
**não coberta / oráculo / servidor / integrada**, o cenário do SEI e o
teste diferencial faltante. Registrar a lacuna **não bloqueia** uma PR
prioritária do IES. Preserve a reserva da rotina e abra ou referencie uma
issue de emulação independente para a fatia seguinte.
Quando as dependências estiverem na main, 🐘 libera a próxima issue
com a etiqueta `handoff-chatgpt`; nunca antes.

Uma capacidade só pode ser chamada **integrada** depois de resposta
servida pelo anti-IES, interpretada pelo caminho **real** do IES e
conferida por oráculo SEI/cassette **independente**, inclusive recusas,
GET/POST e efeitos observáveis. Sucesso sintético produzido e esperado
pelo mesmo código não prova equivalência. O anti-IES devolve recusa
explícita para operações sem evidência, sem simular sucesso.

O IES de produção mantém HTTPS, origem autorizada, cookies privados e
ausência de retry; qualquer transporte local alternativo é apenas de
teste. Sem acesso a produção; testes live seguem autorização apenas de
treinamento. A ordem e o gate da primeira operação estão no
[roadmap conjunto](docs/anti-ies/roadmap.md) e na
[matriz anti-IES](docs/anti-ies/cobertura.md).

## Método: observar → contratar → RED → GREEN → validar

1. **Observar.** Toda capacidade parte de comportamento real: cassettes,
   capturas privadas ou experimento ao vivo numa instalação de
   treinamento. Leia a tela pela **estrutura** (ids, campos, JS servido).
   Não invente operação, campo ou regra que a tela não mostrou; uma
   instalação não prova as outras.
2. **Contratar.** Contrato OKF da operação, erros tipados, shim compilável
   e [decisão por operação](docs/decisoes/unidade-listar-processos.md)
   (alternativas, evidência, trade-offs, revisão; o gate `check_design`
   confere a ligação, não o mérito).
3. **RED.** Testes com fixtures **sintéticas** que reproduzem a estrutura
   observada, executados falhando antes do código. Registre a contagem.
4. **GREEN.** Implementação mínima, refatoração e gates.
5. **Validar ao vivo.** Em treinamento, com orçamento de requests
   declarado; registre PASS/FAIL/NOT_RUN no
   [relatório live](docs/validacao-live/). Sem execução não há PASS:
   `NOT_RUN` com o bloqueio explícito.

O que falta implementar e em que ordem está na
[matriz de derivação](docs/matriz-derivacao-cli.md).

## Princípios de design

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
  O erro é declarado perto do ponto que o levanta. Novas fatias usam o
  próprio tipo na fronteira Serde, sem subcódigos ou tabela de nomes
  paralela. `Causa` permanece para consumidores legados durante a migração.
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

**Fonte normativa integral:** [mecânica e CI](docs/engenharia/mecanica-do-repositorio.md).
Toda rotina implementadora deve ler esse documento antes de editar, testar
ou publicar PR; a extração não altera gates, tamanho, WIP ou limites.

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
