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
