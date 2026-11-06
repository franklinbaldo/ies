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
Testes comparam variantes, não mensagens. Nomes descrevem a condição
observada, evitando qualificadores vagos como `ambíguo` e `divergente`.
O subcódigo `numero_do_historico_diferente_do_solicitado` identifica
a condição observada, sem compromisso artificial com clientes inexistentes.

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

## Partes do guia

- [Erros de transporte e sessão](erros-tipados/transporte-e-sessao.md): bloqueio de segurança, 302 do Controle para o login e leitura coerente da sessão.
- [Erros das leituras](erros-tipados/leituras.md): anexo externo, pesquisa em documentos, versões e filtros da pesquisa.
- [Erros das escritas](erros-tipados/escritas.md): concluir, reabrir, assinar e enviar o processo.
- [Erros locais](erros-tipados/locais.md): configuração local e gravação privada de arquivos.
- [Catálogo de subcódigos](erros-tipados/catalogo.md): catálogo literal por contexto, nomes contextuais e históricos e literais passados por helpers.


## Subcódigos de blocos (#256)

Os seguintes subcódigos discriminam a causa de falha fechada em leitura Web de bloco; links e cookies nunca aparecem na saída.

| Subcódigo | Descrição |
| --- | --- |
| `bloco_nao_encontrado_na_pagina` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `cabecalho_de_blocos_nao_reconhecido` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `cabecalho_do_bloco_invalido` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `faixa_de_blocos_incoerente` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `faixa_do_bloco_invalida` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `html_de_blocos_invalido` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `html_do_bloco_ilegivel` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `legenda_de_blocos_invalida` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `legenda_do_bloco_invalida` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `linha_de_bloco_fora_da_forma` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `linha_do_bloco_invalida` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `link_de_blocos_ausente` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `link_de_blocos_nao_reconhecido` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `links_de_blocos_repetidos` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `numero_do_bloco_invalido` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `numero_do_bloco_repetido` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `pagina_de_blocos_nao_implementada` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `tabela_de_blocos_ausente` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `tabela_do_bloco_ausente` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `tabelas_de_blocos_repetidas` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `tabelas_do_bloco_repetidas` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
| `unidade_inesperada_na_leitura_de_blocos` | Leitura Web de blocos: observação insuficiente, forma inesperada ou opção ainda não implementada; nenhuma escrita ou retry |
