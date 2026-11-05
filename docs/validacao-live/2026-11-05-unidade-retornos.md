---
type: SEIObservation
title: Validacao live pendente de unidade retornos GO e PEN3
description: NOT_RUN sem requisicoes ao SEI no handoff 253
---

# Unidade retornos — validacao live de 05/11/2026

| Instalação de treinamento | Operação | Resultado | Requisições |
| --- | --- | --- | ---: |
| Goiás | `ies unidade retornos --mes 2026-10` | **NOT_RUN** | 0 |
| PEN3 | `ies unidade retornos --mes 2026-10` | **NOT_RUN** | 0 |
| GO e PEN3 | `ies unidade retornos` | **NOT_RUN** | 0 |

Nao foi enviada qualquer requisicao ao SEI nesta sessao:
o sandbox nao tem Rust instalado e nao conseguiu resolver
`github.com` via DNS para obter/buildar o repositorio.
A leitura sintetica e o CI remoto **nao** equivalem ao ensaio live.
A autorizacao da issue e apenas para leitura em treinamento;
nenhuma escrita foi tentada.

## Plano de verificacao

- Compilar `ies` e executar o comando em GO e na PEN3
  usando apenas contas publicas e sessoes privadas `0600`.
- Respeitar 30 s entre comandos e 60 s nas PEN, sem retry.
- Comparar GETs reais com o orcamento: 3 para mes ja aberto,
  mais um por passo de mes. Registrar totais e faixas das duas
  tabelas, sem cookies nem URL assinada.
- Fazer replay das cassettes de `unidade-listar-retornos-*`
  do Todos, inclusive mes vazio e paginacao.
- Se uma instalacao bloquear, parar nela e registrar NOT_RUN,
  sem tentar contornar controle de seguranca.
