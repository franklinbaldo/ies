---
type: CommandContract
title: Relacionar e desfazer relações de processos
operation: processo.relacionar
command: ies processo relacionar NUP --com NUP [--remover] [--confirmar]
effect: write-process-state
implementation: partial
---

# Relacionar processos (#258)

**Nesta PR, somente prévia Web está implementada.** O --confirmar
recusa antes de abrir a sessão ou enviar qualquer requisição, com subcódigo
confirmacao_de_relacionamento_nao_validada.

Abertura com a unidade da sessão exigida: a barra do nó PROCESSO fornece
procedimento_relacionar como URL assinada. GET da tela de relacionamentos,
um formulário frmProcedimentoListarRelacionamentos e uma tabela com summary
Lista de Processos Relacionados. Só os links das linhas da tabela comprovam
vínculos. Uma linha repetida ou uma tabela ausente/repetida é recusa.

**Prévia de adicionar:** a consulta AJAX protocolo_RI1023 declarada na tela
recebe idProcedimento com o destino informado. O XML deve publicar
IdProcedimento decimal e ProtocoloProcedimentoFormatado, ambos únicos.
Não relaciona consigo mesmo. Id já presente na tabela retorna sem_efeito.
Orçamento: abertura + 1 GET da tela + 1 POST de consulta (leitura lógica).

**Prévia de remover:** encontra linha pelo NUP informado e exige
acaoRemover('id'); na mesma linha. Sem relacionamento, sem_efeito true.
Orçamento: abertura + 1 GET da tela, sem POST.

A escrita futura requer um POST derivado da tela servida, uma única vez,
sem seguir 302 e com releitura independente da mesma lista. Efeito incerto
sai 3. A remoção requer a action procedimento_excluir_relacionamento
observada dentro de acaoRemover, nunca texto solto. Não é implementada
enquanto faltarem provas de escrita única e de verificação.

Campos de sucesso JSON: nup, id_procedimento, destino, id_destino (null
quando não observado), remover, confirmado false, alterado false,
sem_efeito, origem web e requisicoes. Nunca expor URL, infra_hash, cookie,
senha ou argumento secreto. Exit 0 prévia, 1 recusa, 2 uso inválido.

## Evidência

- Todos @323eb01: src/todos/processos.py::relacionar,
  tests/test_relacionar_010.py, tests/test_processo_relacionar_remover_010.py,
  tests/paginas_reais/procedimento_relacionar.html.
- Cassette PEN1 processo-relacionar-remover-pen1: 25 interações agregadas;
  replay completo IES NOT_RUN.
- Testes sintéticos src/processo_relacionar/tests.rs.
- RED local NOT_RUN; CI a verificar. Live PEN1/UnB NOT_RUN (0 pedidos);
  escrita NOT_RUN, resíduo nenhum.
