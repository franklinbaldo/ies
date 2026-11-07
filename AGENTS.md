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
6. A unidade de progresso é a **capacidade da issue**, não a rodada
   horária, o commit ou a PR. Retome o comportamento ainda pendente e
   avance o máximo seguro; não pare apenas porque houve push ou CI verde.
   Evite fatias artificiais: scripts, testes, refatoração e documentação
   servem à capacidade e aos gates, não são entrega da CLI por si só.
   Não repita ensaios ou verificações já válidos para o mesmo head só para
   demonstrar atividade; head novo continua exigindo CI próprio.
   **Handoff mínimo**, uma vez por avanço material, na issue ou PR
   (vincule a outra, sem duplicar relatório):
   - `FEITO:` efeito demonstrável ou atividade interna realizada.
   - `CLI:` comando e efeito observável, ou `inalterada`.
   - `CONTINUAR:` próxima alteração funcional concreta ou bloqueio real.
   Diff, SHA e CI ficam nos links do GitHub, sem transcrição ritual.
   Evidência de ensaio live segue em `docs/validacao-live/` somente quando
   houve execução; não transforme `NOT_RUN` em progresso. Mantenha WIP
   e reserva até concluir; nunca alegue PASS sem ensaio real.
7. **Implementadores não fazem merge ou fechamento de issues.**
   Não desligue testes para esconder regressões; **🦡 pode remover testes
   em ablação explícita**, com CI válido. Teste pendente usa
   `#[ignore = "wip #N: motivo específico"]`. Escrita incompleta mantém
   `--confirmar` fechado antes da rede.

### Sincronização obrigatória no começo de **cada rodada**

Antes de codificar ou testar, sincronize a branch com a `main` a cada rodada:

1. Identifique issue e PR próprias e consulte SHAs da `main` e do head.
2. **Sem PR:** branch nova a partir da `main` atual.
3. **Com PR:** se atrás, incorpore `main → branch` por merge verdadeiro.
   Resolva conflitos preservando código integrado. **Nunca use rebase ou force-push.**
4. Confira SHAs e registre na PR. CI verde do SHA antigo não valida o novo.
5. Sem atualização segura ou com conflito não resolvido, não sobrescreva
   código alheio; registre o impedimento e investigue.

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

## Revisão e integração

Procedimentos específicos: [🦅 Gavião](team/gaviao.md) e
[🐘 Elefante](team/elefante.md).

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

- Nenhum arquivo texto passa de 20.000 bytes (exceto `.lock`). Quando
  uma mudança deixar um arquivo acima de ~16.000 bytes, **divida-o na
  mesma PR**, por responsabilidade: o original fica com o núcleo e uma
  seção de links para as partes, e cada parte vira um arquivo próprio
  com frontmatter OKF (`contracts/cli.md` → `contracts/cli/*.md`,
  `docs/erros-tipados.md` → `docs/erros-tipados/*.md`). Não encolha
  texto, tabela ou código para caber; não espere o gate falhar.
- Ao trabalhar em paralelo com outros agentes, edite só os arquivos do
  seu prefixo; os dos outros você lê. Se uma peça compartilhada precisar
  mudar, faça a mudança mínima e explique na PR.
- `xtask` só tem verificadores de infraestrutura, sem dependências.
- **Sem Rust no sandbox, o CI é o compilador.** Faça push no branch da
  PR e leia o resultado do [CI](.github/workflows/ci.yml):
  - **RED:** faça push dos testes novos antes do código. O hook "Suíte
    Rust" falha, e o log mostra quantos testes falharam. Esse é o RED
    observado; registre o run e a contagem.
  - **Binário:** o job "binário ies" publica o artifact
    `ies-linux-x86_64-<sha>` (`ies` em release e `ies.sha256`, 7 dias),
    compilado no ubuntu-22.04 (glibc ≥ 2.35, libssl3). Baixe-o pelo
    GitHub (API de artifacts do run ou `gh run download`), confira o
    SHA-256 e rode no sandbox: comandos offline, replay e ensaio ao vivo
    em treinamento.
  - **Sandbox sem rede para o GitHub ou para o SEI:** rode o ensaio num
    workflow temporário (`workflow_dispatch` ou `push` em `codex/**`) que
    use esse binário, e remova o workflow antes de a PR sair de DRAFT.
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
  (o [CI](.github/workflows/ci.yml) roda os mesmos gates; não crie outro
  conjunto de checks).

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
