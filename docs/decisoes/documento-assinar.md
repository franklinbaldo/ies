---
type: ImplementationDecision
title: Assinatura com a senha do login pela tela que a árvore publica, verificada pela caneta da árvore
operation: documento.assinar
implementation: partial
status: adopted-for-scope
scope: ies documento assinar DOC com prévia (abertura até a árvore, GET da tela do documento e da tela de assinatura, sem POST) e --confirmar com --cargo exato, um POST de frmAssinaturas com a senha do arquivo privado e verificação pela caneta ASSINATURA de uma leitura nova da árvore
selected_strategy: web-tela-de-assinatura-verificada-pela-caneta
alternatives: web-tela-de-assinatura-verificada-pela-caneta,web-verificada-pela-impressa,web-cargo-sugerido-pela-tela,rest-wssei
evidence_level: cassettes-publicas-sete-instalacoes-e-testes-sinteticos
live_validation: previa-pen1-0504-escrita-not-run-aguarda-autorizacao
---

# Assinar o documento (Refs #174)

## Contexto

Primeira escrita no documento, e a única que **não se desfaz**. A barra do
nó `DOCUMENTO` publica `assinarDocumento()`; a URL está na tela do
documento (função de três linhas no 4.0, `var linkAssinarDocumento` no
4.1/5.0); a tela `frmAssinaturas` pede cargo e senha (observação em
[assinatura do documento](../como-o-sei-funciona/assinatura-do-documento.md)).
Nas 77 assinaturas gravadas nas sete instalações, a resposta foi a própria
tela com `finalizar()` e a árvore seguinte mostrou a caneta do assinante
com o cargo escolhido.

## Alternativas consideradas

1. **Tela de assinatura, verificada pela caneta da árvore** (escolhida):
   a leitura que já abre o documento lê a caneta `ASSINATURA`, que tem a
   mesma forma nas sete instalações (núcleo do SEI).
2. **Verificada pela página impressa**, como o Todos faz primeiro: o
   carimbo "Documento assinado eletronicamente por…" muda de texto por
   instalação (o Todos precisou da caneta como reserva, #1148) e custa um
   GET a mais.
3. **Cargo sugerido pela tela** quando `--cargo` falta, como o Todos: menos
   digitação, mas o cargo vai impresso numa assinatura que não se desfaz; a
   prévia mostra a sugestão e `--confirmar` exige o cargo por extenso.
4. **REST/WSSEI**: fora da prioridade ([ADR 0001](../arquitetura/0001-rotas-por-evidencia.md)).

## Comparação

| Critério | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| Instalações cobertas pela evidência | 7 | 7 (texto varia) | 7 | 0 |
| Requisições da escrita a frio (`--processo`) | 6 GET + 1 POST | +1 GET | 6 + 1 | — |
| Cargo escolhido explicitamente | sim | sim | não | — |
| URL montada pelo IES | nenhuma | nenhuma | nenhuma | — |

## Decisão

- A abertura é a de `ies documento DOC` **até a árvore**, sem o GET do
  conteúdo (o leitor genérico `FimDaLeitura` dos três caminhos ganhou o fim
  `SoArvore`).
- Os GETs da tela do documento e da tela de assinatura entram na prévia:
  nas sete instalações a árvore relida depois deles não mostra assinatura,
  e só a tela traz o assinante e os cargos.
- **Senha**: a linha `senha=` do arquivo `0600` de `IES_CREDENCIAIS_FILE`
  (a do login), lida só com `--confirmar` e antes da rede; só entra no corpo
  do POST (o `Debug` da submissão e das credenciais é redigido). **Cargo**:
  `--cargo` igual ao `value` de uma opção servida; não vai ao arquivo, cujo
  formato é fechado em três chaves.
- Documento já assinado pelo usuário da tela: nada a fazer (saída 0).
  Assinado por outro: recusa, porque a dica com dois assinantes não foi
  observada e a segunda assinatura não poderia ser conferida.
- Uma escrita, nunca repetida; a resposta sem `finalizar()` não decide:
  a leitura reconcilia, e só "mesmas assinaturas" é não aplicada (1); o
  resto é escrita não confirmada (3).

## Evidências verificáveis

- [Contrato](../../contracts/cli/documento-assinar.md) e
  [evidência da operação](../../contracts/evidencias/documento-assinar.md).
- Testes: `src/arvore_processo/tests_assinatura.rs` (ação, caneta nas duas
  formas, tela do documento, link nas duas formas),
  `src/documento_assinar/tests_tela.rs`, `src/documento_assinar/tests_fluxo.rs`
  (sequências exatas: prévia 5, escrita 7) e a superfície em
  [cli_documento_assinar.rs](../../tests/cli_documento_assinar.rs).
- Replay do acervo (`documento_assinar::acervo`, `#[ignore]`): 39
  cassettes; 86 telas conferidas e derivadas (11 campos), 77 respostas com
  `finalizar()`, 99 verificações pela caneta, nenhuma recusa.
- Live: [relatório](../validacao-live/2026-11-04-documento-assinar.md).

## Limitações e revisão

- A escrita não foi ensaiada ao vivo (exige autorização do proprietário:
  a assinatura não se desfaz); o plano está no relatório.
- A recusa da senha errada não foi gravada: ela sai como resposta sem
  `finalizar()` e a leitura decide (`assinatura_nao_aplicada`).
- Certificado digital, outro assinante, blocos e vários documentos ficam
  fora. Rever se uma instalação publicar outra forma de link, de caneta ou
  de resposta.
