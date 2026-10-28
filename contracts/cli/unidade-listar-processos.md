---
type: CommandContract
title: Listar processos da unidade
operation: unidade.listar_processos
command: ies unidade listar-processos
effect: read
implementation: partial
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
---

# Listar processos da unidade

`ies unidade listar-processos` lê as tabelas `Recebidos` e `Gerados`
na caixa da unidade ativa. A implementação Rust possui **primeira leitura
de página**, por cassette verificada ou GET HTTPS de uma sessão existente.

## Entradas

- `--cassette-card ARQ.md --zip-file ZIP`: replay offline, com hashes do
  card OKF e validação da instalação. `--passo N` seleciona uma resposta; pode ser repetido para consolidar
  páginas já capturadas (GET inicial; demais respostas GET ou POST offline).
- `--url-caixa-file ARQ --cookie-file ARQ`: leitura de URL assinada
  **já obtida da sessão**; nenhuma autenticação é iniciada.
- `--saida ARQ`: grava JSON em arquivo novo, `0600` em Unix.
- `--json`: imprime JSON de resultado (também pode gravar em arquivo).
- `--unidade SIGLA`: rótulo opcional de unidade informado pelo usuário,
  não prova por si só qual unidade o servidor usou.

Sem origem explícita, permanece `nao_implementado` até existir
a configuração e autenticação canônicas do IES. Entradas incompletas
ou misturadas são erro de uso.

## Resultado de domínio e contrato JSON v1

O domínio tipado guarda apenas fatos: `Caixa` possui duas
`Tabela<Processo>`, cada qual com `processos` (id interno validado,
número e visualização) e `total_declarado: Option<usize>`.

A paginação verifica intervalos e identidades, sem armazenar
`completa`, contagens redundantes ou uma enumeração de completude
dentro do domínio.

O JSON v1 expõe `unidade`, `origem`, `recebidos` e `gerados`.
Cada tabela inclui apenas `processos`, `total_declarado` e
`completa: bool`, **calculado a partir dos intervalos verificados**.
`false` quer dizer cobertura não comprovada, não necessariamente
incompletude definitiva. Não existem no novo contrato os campos
antigos `completude`, `linhas_lidas`, `paginas_lidas`, `avisos`
e `fonte`. A saída humana calcula seus avisos separadamente.

O [golden JSON v1](../../tests/fixtures/caixa-v1.json)
protege o formato público. Não há campo `href` no `Processo`
para permitir serializar URLs assinadas inadvertidamente.

## Segurança e verificação

O HTTP permite somente GET HTTPS da ação `procedimento_controlar`
publicada com `infra_hash`; não segue redirect nem faz retries.
O modo offline confere os hashes do ZIP e do JSON, schema e origem.
O parser recusa tabela ausente, ID repetido e número ambíguo.

Esta fase **consolida páginas offline**, exigindo faixas contíguas,
totais coerentes e IDs distintos por categoria. A paginação HTTP
continua bloqueada: o corpus histórico não preserva os corpos de POST
necessários para reproduzir esse gesto com segurança. Se só a primeira
página foi lida, a saída continua explicitamente parcial.

[Evidências](../evidencias/unidade-listar_processos.md),
[replay](../../docs/cassettes.md) e
[como o SEI funciona](../../docs/como-o-sei-funciona/tabelas-e-paginacao.md)
sustentam este recorte. Ainda não há equivalência funcional plena
com a CLI do Todos.
