---
type: Guide
title: Erros tipados no IES
description: Condições específicas, códigos estáveis e tratamento seguro no Rust
---

# Erros tipados

O IES adota a disciplina do [Todos](https://github.com/franklinbaldo/todos/blob/main/docs/agentes/erros.md):
cada condição semanticamente distinta tem identidade tipada; mensagem
não determina lógica de tratamento. Em Rust, variantes de `enum` substituem
a hierarquia de subclasses Python quando a família é fechada.

## Regras

- Declare erro junto à operação ou adaptador que o produz; não faça
  de falhas técnicas entidades administrativas do SEI.
- Use variante específica por condição, com campos tipados quando
  contexto for necessário. Não construa uma variante genérica com
  mensagem variável para simular casos diferentes.
- `subcodigo` identifica a condição de modo estável; a mensagem
  apresentada na CLI pode evoluir sem quebrar automações.
- Preserve o erro concreto até a borda de apresentação. Não inspecione
  o texto para tomar decisões; evite `String` como tipo de erro
  para condições conhecidas.
- A categoria pública descreve o próximo passo do chamador.
  Não criar um grande `ErroIES` para todos os módulos.
- Escritas com resultado incerto nunca recebem retry ou troca de
  transporte automática: verificar por leitura antes de repetir.
- Bugs de programação não devem ser convertidos silenciosamente em
  erros esperados de negócio.

## Primeiro recorte aplicado

`src/historico.rs` devolve `ErroHistorico` com variantes como
`NumeroDoHistoricoDiferenteDoSolicitado`, `HistoricoAusente` e
`LinhaInvalida`. As funções não devolvem mais códigos como
`&'static str`; o texto público deriva de `subcodigo()` na CLI.
Testes comparam variantes, não mensagens. Nomes descrevem a condição\nobservada, evitando qualificadores vagos como `ambíguo` e `divergente`.\nO subcódigo `numero_do_historico_diferente_do_solicitado` identifica\na condição observada, sem compromisso artificial com clientes inexistentes.

`ErroCaixa` e `FalhaTransporte` já utilizam variantes específicas.
Revisões posteriores devem acrescentar dados úteis e padronizar
categoria e apresentação, sem alterar códigos públicos desnecessariamente.

## Dados de apresentação nas recusas HTTP

`FalhaTransporte` carrega `StatusHttpRecusado` (status e mensagem
saneada do SEI) em 3xx, 401/403, 429, 5xx e outro status; por isso
deixou de ser `Copy`. A variante e o subcódigo continuam sendo a
classificação: o status e a mensagem são só apresentação e nenhum
código os inspeciona para decidir tratamento. Ver o
[contrato da CLI](../contracts/cli.md).

## Bloqueio de segurança da instalação

`FalhaTransporte::BloqueioDeSegurancaDaInstalacao`
(`bloqueio_de_seguranca_da_instalacao`) é o 403 com a página de bloqueio
do WAF de GO (03/11/2026, depois de rajadas): `<title>Acesso Negado</title>`
e a frase "bloqueada por nossa política de segurança" no texto visível
do corpo; ou a do Cloudflare da PEN2 (04/11/2026, segundo GET de
`ies login`): `<title>Attention Required! | Cloudflare</title>` e
"Sorry, you have been blocked". É reconhecimento de **tela** (como o da tela de login), feito
na camada de transporte para todos os comandos Web
(`src/transporte/bloqueio_de_seguranca.rs`), não leitura da mensagem para
decidir tratamento. Expõe `status_http` 403 e a mensagem saneada com o
código de bloqueio (não é segredo; URLs e cookies saem redigidos). A
sessão continuou válida: não há relogin, retry nem pausa automática. O
403 sem essa página continua `sessao_nao_autorizada`. Espace as execuções
ao vivo ([estado e instalações](como-o-sei-funciona/estado-e-instalacoes.md)).
