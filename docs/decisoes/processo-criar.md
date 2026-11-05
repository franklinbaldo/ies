---
type: ImplementationDecision
title: Criar processo pela escolha e cadastro publicados pelo SEI
operation: processo.criar
implementation: not-started
status: proposed
scope: escolha-de-tipo-e-cadastro-web-com-previa-e-verificacao-por-leitura
selected_strategy: web-formulario-servido-e-releitura-da-arvore
alternatives: web-formulario-servido-e-releitura-da-arvore,post-montado-estaticamente,rest-wssei
evidence_level: todos-e-captura-sei-409-sem-replay-ies
live_validation: not-run
---

# Decisão proposta: iniciar processo (issue #245)

## Contexto e alternativas

O oráculo Todos demonstra criação em instalações de treinamento.
A lista de tipos pode ser parcial; o cadastro e o modo de escolha
dependem da versão do SEI. O comando desejado é
`ies processo criar --tipo TIPO --especificacao TEXTO`,
com prévia por padrão, `--nivel publico|restrito`,
`--interessado` repetível e `--confirmar`.

1. **Web, formulário servido, confirmação por leitura** (proposta).
   Usa ações e controles observados, uma escrita e a árvore relida;
   compatível com o primitivo `formulario_web`.
2. POST montado de campos fixos: menos passos, porém incapaz
   de garantir que o tipo, a hipótese e os hidden sejam oferecidos.
3. REST/WSSEI: sem prioridade no IES e não demonstrado no oráculo.

## Decisão e limites

Seguir a alternativa 1, **após** implementar os testes RED.
Escolher exclusivamente um tipo demonstrável pela lista completa,
abrir o cadastro pela ação publicada e conferir campos,
assunto sugerido, nível e hipótese antes de escrever.
Na prévia, apresentar o plano e os campos efetivamente lidos,
sem POST criador. O POST de escolha de tipo, quando publicado
pela versão, somente abre o cadastro.

Com `--confirmar`, o primitivo deve enviar uma única vez o
formulário derivado, com efeito avaliado de criação de processo.
Desabilitar redirecionamento automático; obter identidade da
resposta e reler a árvore. Falha posterior à escrita cuja leitura
não confirme a identidade é efeito incerto (3), sem repetição.
Não tratar simples HTTP 200/302 como prova suficiente.

Não usar regex nem executar JavaScript. Extrair atributos
unicamente por `src/html_atributos`; nunca expor URLs assinadas,
cookies ou `infra_hash`. Nenhuma escrita live até a autorização
expressa do proprietário na issue.

## Critérios antes da adoção

- RED: tipo ausente na lista completa, duplicado, incompleta,
  sem ação conhecida, assunto ausente/múltiplo, rádio desabilitado,
  hipótese obrigatória ausente, interessado ambíguo.
- GREEN: prévia com zero POST criador; confirmação com exatamente
  um POST criador e verificação fresca da identidade.
- Respostas: 302 com ID e NUP coincidentes, 200 com recusa,
  falha de rede depois do POST, 302 sem ID, árvore indisponível.
- Registrar contagem RED antes da implementação; replay de
  cassettes e orçamento por versão; executar gate pre-push e CI.
- Prévia live e escrita live têm registros independentes
  PASS/FAIL/NOT_RUN. Na ausência de execução: NOT_RUN.

## Evidências e revisão

[Observação estrutural](../como-o-sei-funciona/inicio-do-processo.md);
[matriz do Todos](../../contracts/evidencias/processo-criar.md);
[issue #245](https://github.com/franklinbaldo/ies/issues/245).
Rever quando houver divergência no cadastro servido pela UFRJ/UnB
ou um replay que refute o redirecionamento observado pelo Todos.
