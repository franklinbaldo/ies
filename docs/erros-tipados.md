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
Testes comparam variantes, não mensagens. Nomes descrevem a condição\nobservada, evitando qualificadores vagos como `ambíguo` e `divergente`.\nO subcódigo público legado `processo_divergente` permanece por compatibilidade\naté migração explícita do contrato; o tipo Rust é descritivo.

`ErroCaixa` e `FalhaTransporte` já utilizam variantes específicas.
Revisões posteriores devem acrescentar dados úteis e padronizar
categoria e apresentação, sem alterar códigos públicos desnecessariamente.
