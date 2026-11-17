---
type: Guide
title: Validação da leitura e escrita de comentários do processo na UnB
description: Estado NOT_RUN e orçamento das operações da issue 257
---

# Processo comentários / comentar — 05/11/2026

**Instalação:** UnB, exclusivamente treinamento.
**Estado:** NOT_RUN (sem sessão ou ambiente de rede SEI acessível nesta execução).
**Requisições:** 0 GET, 0 POST, total 0.
**Prévia:** NOT_RUN. **Escrita ao vivo:** NOT_RUN.
**Resíduo no SEI:** nenhum (nenhuma alteração enviada).

Evidência inspecionada: fonte do Todos `src/todos/comentarios.py`
e `tests/test_comentarios_010.py`, cassettes públicas pelos cards
[UnB](../../cassettes/unb/comentarios-ciclo-unb--c7d0c8a53b5e.md),
[UFRJ](../../cassettes/ufrj/comentarios-ciclo-ufrj--6a443c99186b.md)
e [GO](../../cassettes/go/comentarios-ciclo-go--76c984b41a29.md).
Replay dos ZIPs: NOT_RUN; acesso ao Internet Archive bloqueado
por resolução DNS no sandbox. A análise estrutural do oráculo não
é PASS em uma instalação.

## Etapas pendentes

1. Replay offline dos HTMLs registrados nas cassettes, comparando
   versões SEI 4.x e 5.x com o mesmo leitor de produção.
2. CLI Web de leitura: abertura de árvore por 1–4 requisições
   conforme índice; GET único de `comentario_listar`.
   Registrar contagem observada, sem repetir requisições.
3. Prévia Web de comentar: obter cadastro sem efetuar POST de gravação.
4. Após testes integrados de POST único e verificação por leitura:
   no treinamento, executar **uma única** confirmação após prévia
   e reler. Registrar comentário criado (id e conteúdo não sensível),
   inclusive eventual resíduo, sem publicar URL assinada, cookie,
   senha nem `infra_hash`.
