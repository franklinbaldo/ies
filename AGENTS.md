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
   treinamento. Leia a tela pela **estrutura** (ids, campos, JS servido),
   nunca copie valores. Não invente operação, campo ou regra que a tela
   não mostrou; uma instalação não prova as outras.
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
- Senhas, cookies, URLs assinadas, `infra_hash` e HTML autenticado nunca
  vão para argumentos, logs, commits, fixtures ou saídas. Sessão e
  credenciais só em arquivos privados `0600`.
- GET também pode ter efeito (abrir processo registra recebimento);
  avalie o efeito de toda requisição antes de enviá-la.

## Mecânica do repositório

- Todo Markdown autoral é OKF ([política de CI](docs/ci.md)); nenhum
  arquivo texto passa de 20.000 bytes — divida por responsabilidade.
- `xtask` só tem verificadores de infraestrutura, sem dependências.
- Antes de qualquer PR:
  `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files`
  (o [CI](.github/workflows/ci.yml) roda os mesmos gates; não crie outro
  conjunto de checks).

## Onde está o resto

[Contratos](contracts/cli.md) · [evidências por operação](contracts/evidencias/README.md)
· [cassettes](docs/cassettes.md) · [como o SEI funciona](docs/como-o-sei-funciona/README.md)
· [decisões](docs/decisoes/) · [acesso por instalação](docs/interfaces/acesso-por-instalacao.md)
· [ablação de fluxos](docs/experimentos/ablacao-fluxos-web.md)
