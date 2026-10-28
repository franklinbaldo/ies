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
  card OKF e validação da instalação. `--passo N` seleciona uma resposta
  quando houver várias caixas na mesma cassette.
- `--url-caixa-file ARQ --cookie-file ARQ`: leitura de URL assinada
  **já obtida da sessão**; nenhuma autenticação é iniciada.
- `--saida ARQ`: grava JSON em arquivo novo, `0600` em Unix.
- `--json`: imprime JSON de resultado (também pode gravar em arquivo).
- `--unidade SIGLA`: rótulo opcional de unidade informado pelo usuário,
  não prova por si só qual unidade o servidor usou.

Sem origem explícita, permanece `nao_implementado` até existir
a configuração e autenticação canônicas do IES. Entradas incompletas
ou misturadas são erro de uso.

## Resultado de domínio

`recebidos` e `gerados` são coleções distintas, cada uma com:
`processos` (`numero`, `id`, `visualizado`),
`total_declarado`, `linhas_lidas` e `completude`.

`completude` é `completa`, `parcial` ou `indeterminada`.
A resposta total informa `parcial_ou_indeterminada` quando
qualquer das duas coleções não está comprovadamente completa.
URLs assinadas nunca são incluídas nos resultados.

## Segurança e verificação

O HTTP permite somente GET HTTPS da ação `procedimento_controlar`
publicada com `infra_hash`; não segue redirect nem faz retries.
O modo offline confere os hashes do ZIP e do JSON, schema e origem.
O parser recusa tabela ausente, ID repetido e número ambíguo.

Esta fase **não pagina**. A página da caixa pode ter um total
maior que as linhas lidas; nesse caso, a saída é explicitamente
parcial. Nenhuma alteração do SEI é efetuada.

[Evidências](../evidencias/unidade-listar_processos.md),
[replay](../../docs/cassettes.md) e
[como o SEI funciona](../../docs/como-o-sei-funciona/tabelas-e-paginacao.md)
sustentam este recorte. Ainda não há equivalência funcional plena
com a CLI do Todos.
